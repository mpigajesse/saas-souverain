--
-- PostgreSQL database dump
--

\restrict YV2eQHyPLom2fVrOsR65DueBtzCC3g9pYdgAbNbdeYPbqAjPi8vPrADuu3MTqkF

-- Dumped from database version 17.11
-- Dumped by pg_dump version 17.11

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: egress_logs; Type: TABLE; Schema: public; Owner: egress
--

CREATE TABLE public.egress_logs (
    id bigint NOT NULL,
    domain text NOT NULL,
    status text NOT NULL,
    latency_ms bigint,
    via_proxy boolean DEFAULT false,
    error_msg text,
    created_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.egress_logs OWNER TO egress;

--
-- Name: egress_logs_id_seq; Type: SEQUENCE; Schema: public; Owner: egress
--

CREATE SEQUENCE public.egress_logs_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.egress_logs_id_seq OWNER TO egress;

--
-- Name: egress_logs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: egress
--

ALTER SEQUENCE public.egress_logs_id_seq OWNED BY public.egress_logs.id;


--
-- Name: schema_version; Type: TABLE; Schema: public; Owner: egress
--

CREATE TABLE public.schema_version (
    version integer NOT NULL,
    applied_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.schema_version OWNER TO egress;

--
-- Name: egress_logs id; Type: DEFAULT; Schema: public; Owner: egress
--

ALTER TABLE ONLY public.egress_logs ALTER COLUMN id SET DEFAULT nextval('public.egress_logs_id_seq'::regclass);


--
-- Data for Name: egress_logs; Type: TABLE DATA; Schema: public; Owner: egress
--

COPY public.egress_logs (id, domain, status, latency_ms, via_proxy, error_msg, created_at) FROM stdin;
1	http://httpbin.org/get	success	244	f		2026-11-25 19:49:17.501192+00
2	https://api.github.com	success	518	f		2026-11-25 19:49:18.028691+00
\.


--
-- Data for Name: schema_version; Type: TABLE DATA; Schema: public; Owner: egress
--

COPY public.schema_version (version, applied_at) FROM stdin;
3	2026-11-25 19:48:51.75637+00
\.


--
-- Name: egress_logs_id_seq; Type: SEQUENCE SET; Schema: public; Owner: egress
--

SELECT pg_catalog.setval('public.egress_logs_id_seq', 2, true);


--
-- Name: egress_logs egress_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: egress
--

ALTER TABLE ONLY public.egress_logs
    ADD CONSTRAINT egress_logs_pkey PRIMARY KEY (id);


--
-- Name: schema_version schema_version_pkey; Type: CONSTRAINT; Schema: public; Owner: egress
--

ALTER TABLE ONLY public.schema_version
    ADD CONSTRAINT schema_version_pkey PRIMARY KEY (version);


--
-- Name: idx_egress_logs_created_at; Type: INDEX; Schema: public; Owner: egress
--

CREATE INDEX idx_egress_logs_created_at ON public.egress_logs USING btree (created_at);


--
-- Name: idx_egress_logs_domain; Type: INDEX; Schema: public; Owner: egress
--

CREATE INDEX idx_egress_logs_domain ON public.egress_logs USING btree (domain);


--
-- Name: idx_egress_logs_status; Type: INDEX; Schema: public; Owner: egress
--

CREATE INDEX idx_egress_logs_status ON public.egress_logs USING btree (status);


--
-- PostgreSQL database dump complete
--

\unrestrict YV2eQHyPLom2fVrOsR65DueBtzCC3g9pYdgAbNbdeYPbqAjPi8vPrADuu3MTqkF

