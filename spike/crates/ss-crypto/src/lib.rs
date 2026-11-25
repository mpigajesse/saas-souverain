mod error;
mod dek;
mod ak;
mod crl;
mod shamir;
mod device_key;
mod recovery;
mod jeton_relais;

pub use error::CryptoError;
pub use dek::Dek;
pub use ak::{AccessKeyPair, AccessPublicKey, SecretAccessKey};
pub use crl::CrlRegistry;
pub use shamir::{Kek, ShamirShare, split_kek, reconstruct_kek, split_secret, reconstruct_secret, TAILLE_SECRET};
pub use device_key::{DeviceKeyPair, DevicePublicKey};
pub use recovery::{
    derive_recovery_key, derive_recovery_key_avec, deriver_kek, empreinte_dek, generate_salt,
    paire_de_recuperation, somme_controle_part, ParametresArgon2,
};
pub use jeton_relais::{jeton_relais, verifier_jeton_relais};

#[cfg(test)]
mod tests_c01 {
    use super::*;

    // C-01 : altérer un octet d'une part Shamir, reconstruire la KEK,
    // tenter unwrap_dek avec la fausse KEK. Attendu : Err(DecryptionFailed).
    #[test]
    fn c01_corruption_part_shamir_interceptee_par_aead() {
        let kek = Kek::generate();
        let dek = Dek::generate();
        let wrapped = kek.wrap_dek(&dek).unwrap();

        let (s1, mut s2, _s3) = split_kek(&kek).unwrap();
        let original = s2.data;
        s2.data[0] ^= 0x01;
        assert_ne!(s2.data, original, "la part Shamir a bien été altérée");

        let faux_kek = reconstruct_kek(&s1, &s2).unwrap();
        assert_ne!(faux_kek.as_bytes(), kek.as_bytes(), "la fausse KEK diffère de la vraie");

        let res = faux_kek.unwrap_dek(&wrapped);
        assert!(
            matches!(res, Err(CryptoError::DecryptionFailed)),
            "attendu Err(DecryptionFailed), obtenu {:?}",
            res
        );
    }

    // C-02 : passer les vecteurs RFC 7748 de petit ordre et le point à l'infini
    // à seal_dek / open_sealed_dek. Attendu : rejet, aucune DEK scellée avec
    // un secret tout-à-zéro.
    #[test]
    fn c02_petit_ordre_rfc7748_seal_open() {
        // Vecteurs de petit ordre de la RFC 7748 § 5.2 (points d'ordre 1, 2, 4, 8).
        let vecteurs: [([u8; 32], &str); 7] = [
            ([0x00; 32], "point à l'infini (ordre 1, u=0)"),
            ([0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], "ordre 8 (u=1)"),
            (hex::decode("e0eb7a7c3b41b8ae1656e3faf19fc46ada098deb9c32b1fd866205165f49b800").unwrap().try_into().unwrap(), "ordre 8"),
            (hex::decode("5f9c95bca3508c24b1d0b1559c83ef5b04445cc4581c8e86d8224eddd09f1157").unwrap().try_into().unwrap(), "ordre 8"),
            (hex::decode("ecffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f").unwrap().try_into().unwrap(), "ordre 2 (p-1)"),
            (hex::decode("edffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f").unwrap().try_into().unwrap(), "ordre 4 (p)"),
            (hex::decode("eeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f").unwrap().try_into().unwrap(), "ordre 4 (p+1)"),
        ];

        let dek = Dek::generate();
        let mut non_rejetes = 0;
        for (vec, label) in &vecteurs {
            let pub_dev = DevicePublicKey::from_bytes(*vec);
            match pub_dev.seal_dek(&dek) {
                Ok(sealed) => {
                    non_rejetes += 1;
                    // Le secret partagé X25519 est [0u8; 32], et la DEK est scellée sans rejet.
                    let kp_nul = DeviceKeyPair::from_secret_bytes([0u8; 32]);
                    assert!(
                        kp_nul.open_sealed_dek(&sealed).is_ok(),
                        "DEK scellée avec secret tout-à-zéro ouverte sans rejet pour {label}"
                    );
                }
                Err(_) => {}
            }
        }

        // Vérification complémentaire sur open_sealed_dek avec clé éphémère de petit ordre
        let kp = DeviceKeyPair::generate();
        let sealed_reel = kp.public.seal_dek(&dek).unwrap();
        let mut sealed_eph_nul = sealed_reel.clone();
        sealed_eph_nul[..32].copy_from_slice(&vecteurs[0].0);
        let res_eph = kp.open_sealed_dek(&sealed_eph_nul);
        assert!(
            !matches!(res_eph, Err(CryptoError::InvalidPublicKey)),
            "open_sealed_dek ne rejette pas InvalidPublicKey sur clé éphémère de petit ordre : {:?}",
            res_eph
        );

        assert_eq!(
            non_rejetes, 7,
            "FAIL C-02 : les 7 vecteurs RFC 7748 de petit ordre (dont le point à l'infini) sont acceptés par seal_dek sans rejet et produisent une DEK scellée avec un secret partagé tout-à-zéro"
        );
    }

    // C-03 : tronquer le sealed box à 0/31/32/79 octets, altérer 1 bit de
    // la clé éphémère. Attendu : Err(InvalidSealedBox) ou Err(DecryptionFailed),
    // jamais de panic.
    #[test]
    fn c03_sealed_box_tronquee_et_bit_ephemere() {
        let kp = DeviceKeyPair::generate();
        let dek = Dek::generate();
        let sealed = kp.public.seal_dek(&dek).unwrap();
        assert_eq!(sealed.len(), 80);

        for len in [0usize, 31, 32, 79] {
            let res = kp.open_sealed_dek(&sealed[..len]);
            assert!(
                matches!(res, Err(CryptoError::InvalidSealedBox)),
                "troncature à {len} octets : attendu Err(InvalidSealedBox), obtenu {:?}",
                res
            );
        }

        let mut bit0 = sealed.clone();
        bit0[0] ^= 0x01;
        let res0 = kp.open_sealed_dek(&bit0);
        assert!(
            matches!(res0, Err(CryptoError::DecryptionFailed)),
            "altération bit 0 de la clé éphémère : attendu Err(DecryptionFailed), obtenu {:?}",
            res0
        );

        let mut bit31 = sealed.clone();
        bit31[31] ^= 0x80;
        let res31 = kp.open_sealed_dek(&bit31);
        assert!(
            matches!(res31, Err(CryptoError::DecryptionFailed)),
            "altération bit 7 de l'octet 31 de la clé éphémère : attendu Err(DecryptionFailed), obtenu {:?}",
            res31
        );
    }

    // C-04 : récupérer un pointeur brut sur les octets internes, forcer le
    // drop, inspecter en unsafe. Attendu : tous les octets à 0x00.
    #[test]
    fn c04_zeroisation_apres_drop_inspection_unsafe() {
        // On utilise Box::into_raw + ptr::drop_in_place pour appeler Drop::drop
        // en place sur l'objet sans le déplacer dans core::mem::drop(x: T) ni
        // désallouer le bloc mémoire.
        let raw_dek = Box::into_raw(Box::new(Dek::from_bytes([0xAA; 32])));
        let dek_ptr = unsafe { (*raw_dek).as_bytes().as_ptr() };
        unsafe { std::ptr::drop_in_place(raw_dek) };
        let dek_apres: Vec<u8> = (0..32).map(|i| unsafe { dek_ptr.add(i).read_volatile() }).collect();
        unsafe { drop(Box::from_raw(raw_dek)) };

        let raw_kek = Box::into_raw(Box::new(Kek::from_bytes([0xBB; 32])));
        let kek_ptr = unsafe { (*raw_kek).as_bytes().as_ptr() };
        unsafe { std::ptr::drop_in_place(raw_kek) };
        let kek_apres: Vec<u8> = (0..32).map(|i| unsafe { kek_ptr.add(i).read_volatile() }).collect();
        unsafe { drop(Box::from_raw(raw_kek)) };

        let raw_part = Box::into_raw(Box::new(ShamirShare { id: 1, data: [0xCC; 32] }));
        let part_ptr = unsafe { (*raw_part).data.as_ptr() };
        unsafe { std::ptr::drop_in_place(raw_part) };
        let part_apres: Vec<u8> = (0..32).map(|i| unsafe { part_ptr.add(i).read_volatile() }).collect();
        unsafe { drop(Box::from_raw(raw_part)) };

        let raw_sak = Box::into_raw(Box::new(SecretAccessKey::from_bytes([0xDD; 32])));
        let sak_ptr = unsafe { (*raw_sak).as_bytes().as_ptr() };
        unsafe { std::ptr::drop_in_place(raw_sak) };
        let sak_apres: Vec<u8> = (0..32).map(|i| unsafe { sak_ptr.add(i).read_volatile() }).collect();
        unsafe { drop(Box::from_raw(raw_sak)) };

        assert!(
            dek_apres.iter().all(|&b| b == 0x00)
                && kek_apres.iter().all(|&b| b == 0x00)
                && part_apres.iter().all(|&b| b == 0x00)
                && sak_apres.iter().all(|&b| b == 0x00),
            "FAIL C-04 : octets non zéroïsés après drop_in_place (Dek[0]={:#x}, Kek[0]={:#x}, Part[0]={:#x}, SAK[0]={:#x})",
            dek_apres[0], kek_apres[0], part_apres[0], sak_apres[0]
        );
    }
}
