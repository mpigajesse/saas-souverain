package whitelist

import (
	"context"
	"log/slog"
)

// Action représente une action que le Copilote IA est autorisé à proposer et
// éventuellement exécuter. Toute action non explicite dans cette liste est
// systématiquement rejetée par construction (même si le LLM en fait la demande).
type Action struct {
	// Name est le nom de l'action qui sera passé au exécuteur.
	Name string
	// RequiresConfirmation indique si une confirmation explicite de l'utilisateur
	// est requise avant exécution. Le Copilote doit afficher "Confirmez-vous ?"
	// [o/n]" et n'agir que si la réponse est 'o' ou 'O'.
	RequiresConfirmation bool
	// Execute est la fonction Go qui effectuera l'action lorsqu'elle sera appelée.
	// Elle ne sera appelée que si la validation de la liste blanche a réussi
	// ET (le cas échéant) l'utilisateur aura confirmé.
	Execute func(ctx context.Context, args map[string]any) error
}

// Whitelist est la liste restreinte d'actions autorisées. Ne jamais y ajouter
// d'actions "lourdes" (redémarrage machine, rotation de clé, etc.) sans
// validation de sécurité approfondie.
// Tout changement de cette liste doit faire l'objet d'une revue de sécurité.
var Whitelist = map[string]Action{
	"restart_service": {
		Name:                 "redémarrer un service applicatif",
		RequiresConfirmation: true,
		Execute:              restartService,
	},
	"clean_wal_logs": {
		Name:                 "nettoyer les logs WAL selon politique prédéfinie",
		RequiresConfirmation: false,
		Execute:              cleanWALLogs,
	},
	// NOTE : aucune action touchant aux clés (rotation, récupération Shamir,
	// redémarrage machine) n'est incluse ici. Elles sont exclues par principe
	// et doivent faire l'objet d'escalade humaine seule.
}

// Validate vérifie si la proposition donnée correspond à une action whitelistée.
// Retourne l'action validée et true si elle est autorisée, ou Action{} et false
// si l'action est inconnue ou non whitelistée.
// Cette fonction est la seule porte d'entrée que le Copilote doit utiliser ;
// jamais d'appel direct Execute().
//
// Critère M2 : 100% des actions non whitelistées rejetées, y compris tentatives
// adverses explicites (ex. prompt qui essaie de faire proposer rm -rf, un redémarrage
// non whitelisté, etc.).
func Validate(name string) (Action, bool) {
	action, ok := Whitelist[name]
	return action, ok
}

// IsNameWhitelist vérifie si un nom d'action fait partie de la liste blanche,
// sans retourner l'action elle-même. Utile pour un rejet silencieux.
func IsNameWhitelist(name string) bool {
	_, ok := Whitelist[name]
	return ok
}

// restart_service est l'exécution réelle du redémarrage de service.
// Elle est référencée dans la whitelist mais n'est appelée que via
// la logique de confirmation du Copilote.
func restartService(ctx context.Context, args map[string]any) error {
	// TODO : implémenter le vrai redémarrage via le service local ou systemctl.
	// Pour l'instant, journalisation uniquement.
	_logger := ctx.Value("logger").(*slog.Logger)
	_logger.Info("redémarrage service demandé (via Copilote)", "args", args)
	return nil
}

// clean_wal_logs est l'exécution réelle du nettoyage des logs WAL.
func cleanWALLogs(ctx context.Context, args map[string]any) error {
	// TODO : implémenter le vrai nettoyage WAL.
	_logger := ctx.Value("logger").(*slog.Logger)
	_logger.Info("nettoyage WAL demandé (via Copilote)", "args", args)
	return nil
}

// ProposeAction représente une proposition du LLM qui doit être validée.
// Cette structure est retournée par le Copilote avant exécution.
type ProposeAction struct {
	// Name est le nom de l'action proposé par le LLM.
	Name string
	// Args sont les arguments fournis par le LLM (peuvent être nil).
	Args map[string]any
	// Valid indique si l'action a passé la validation de la liste blanche.
	Valid bool
	// Reason est la raison du rejet, le cas échéant.
	Reason string
}

// ValidateProposal valide une proposition du LLM. Si le nom de l'action n'est pas
// dans la whitelist, retourne ProposeAction{Valid: false, Reason: "action non autorisée"}.
// Si l'action est whitelistée, retourne ProposeAction{Valid: true, ...}.
func ValidateProposal(name string, args map[string]any) ProposeAction {
	if !IsNameWhitelist(name) {
		return ProposeAction{
			Valid:  false,
			Reason: "action non autorisée : '" + name + "' n'est pas dans la liste blanche",
		}
	}
	return ProposeAction{
		Valid: true,
		Name:  name,
		Args:  args,
		// Execute pourra être appelée plus tard par la logique du Copilote
		// seulement si l'utilisateur confirme.
	}
}
