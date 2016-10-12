--
-- PostgreSQL database dump
--

-- Dumped from database version 9.5.2
-- Dumped by pg_dump version 9.5.2

SET statement_timeout = 0;
SET lock_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SET check_function_bodies = false;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: plpgsql; Type: EXTENSION; Schema: -; Owner: 
--

CREATE EXTENSION IF NOT EXISTS plpgsql WITH SCHEMA pg_catalog;


--
-- Name: EXTENSION plpgsql; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION plpgsql IS 'PL/pgSQL procedural language';


SET search_path = public, pg_catalog;

SET default_tablespace = '';

SET default_with_oids = false;

--
-- Name: activities; Type: TABLE; Schema: public; Owner: aeglbot
--

CREATE TABLE activities (
    id integer NOT NULL,
    name text NOT NULL,
    mode text,
    min_fireteam_size integer NOT NULL,
    max_fireteam_size integer NOT NULL,
    min_light integer,
    min_level integer
);


ALTER TABLE activities OWNER TO aeglbot;

--
-- Name: activities_id_seq; Type: SEQUENCE; Schema: public; Owner: aeglbot
--

CREATE SEQUENCE activities_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE activities_id_seq OWNER TO aeglbot;

--
-- Name: activities_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: aeglbot
--

ALTER SEQUENCE activities_id_seq OWNED BY activities.id;


--
-- Name: old_users; Type: TABLE; Schema: public; Owner: aeglbot
--

CREATE TABLE old_users (
    id integer NOT NULL,
    telegram_name text NOT NULL,
    telegram_id integer NOT NULL,
    psn_name text,
    email text,
    psn_clan text,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL,
    deleted_at timestamp without time zone,
    tokens jsonb,
    pending_activation_code text
);


ALTER TABLE old_users OWNER TO aeglbot;

--
-- Name: plannedactivities; Type: TABLE; Schema: public; Owner: aeglbot
--

CREATE TABLE plannedactivities (
    id integer NOT NULL,
    author_id integer NOT NULL,
    activity_id integer NOT NULL,
    details text,
    start timestamp without time zone NOT NULL
);


ALTER TABLE plannedactivities OWNER TO aeglbot;

--
-- Name: plannedactivities_id_seq; Type: SEQUENCE; Schema: public; Owner: aeglbot
--

CREATE SEQUENCE plannedactivities_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE plannedactivities_id_seq OWNER TO aeglbot;

--
-- Name: plannedactivities_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: aeglbot
--

ALTER SEQUENCE plannedactivities_id_seq OWNED BY plannedactivities.id;


--
-- Name: plannedactivitymembers; Type: TABLE; Schema: public; Owner: aeglbot
--

CREATE TABLE plannedactivitymembers (
    id integer NOT NULL,
    planned_activity_id integer NOT NULL,
    user_id integer NOT NULL,
    added timestamp without time zone DEFAULT '2016-10-12 23:06:36.916'::timestamp without time zone NOT NULL
);


ALTER TABLE plannedactivitymembers OWNER TO aeglbot;

--
-- Name: plannedactivitymembers_id_seq; Type: SEQUENCE; Schema: public; Owner: aeglbot
--

CREATE SEQUENCE plannedactivitymembers_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE plannedactivitymembers_id_seq OWNER TO aeglbot;

--
-- Name: plannedactivitymembers_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: aeglbot
--

ALTER SEQUENCE plannedactivitymembers_id_seq OWNED BY plannedactivitymembers.id;


--
-- Name: plannedactivityreminders; Type: TABLE; Schema: public; Owner: aeglbot
--

CREATE TABLE plannedactivityreminders (
    id integer NOT NULL,
    planned_activity_id integer NOT NULL,
    user_id integer NOT NULL,
    remind timestamp without time zone NOT NULL
);


ALTER TABLE plannedactivityreminders OWNER TO aeglbot;

--
-- Name: plannedactivityreminders_id_seq; Type: SEQUENCE; Schema: public; Owner: aeglbot
--

CREATE SEQUENCE plannedactivityreminders_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE plannedactivityreminders_id_seq OWNER TO aeglbot;

--
-- Name: plannedactivityreminders_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: aeglbot
--

ALTER SEQUENCE plannedactivityreminders_id_seq OWNED BY plannedactivityreminders.id;


--
-- Name: users; Type: TABLE; Schema: public; Owner: aeglbot
--

CREATE TABLE users (
    id integer NOT NULL,
    telegram_name text NOT NULL,
    telegram_id integer NOT NULL,
    psn_name text NOT NULL,
    email text,
    psn_clan text,
    created_at timestamp without time zone DEFAULT '2016-10-12 23:06:36.582'::timestamp without time zone NOT NULL,
    updated_at timestamp without time zone DEFAULT '2016-10-12 23:06:36.6'::timestamp without time zone NOT NULL,
    deleted_at timestamp without time zone,
    tokens text,
    pending_activation_code text
);


ALTER TABLE users OWNER TO aeglbot;

--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: aeglbot
--

CREATE SEQUENCE users_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE users_id_seq OWNER TO aeglbot;

--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: aeglbot
--

ALTER SEQUENCE users_id_seq OWNED BY old_users.id;


--
-- Name: users_id_seq1; Type: SEQUENCE; Schema: public; Owner: aeglbot
--

CREATE SEQUENCE users_id_seq1
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE users_id_seq1 OWNER TO aeglbot;

--
-- Name: users_id_seq1; Type: SEQUENCE OWNED BY; Schema: public; Owner: aeglbot
--

ALTER SEQUENCE users_id_seq1 OWNED BY users.id;


--
-- Name: id; Type: DEFAULT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY activities ALTER COLUMN id SET DEFAULT nextval('activities_id_seq'::regclass);


--
-- Name: id; Type: DEFAULT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY old_users ALTER COLUMN id SET DEFAULT nextval('users_id_seq'::regclass);


--
-- Name: id; Type: DEFAULT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY plannedactivities ALTER COLUMN id SET DEFAULT nextval('plannedactivities_id_seq'::regclass);


--
-- Name: id; Type: DEFAULT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY plannedactivitymembers ALTER COLUMN id SET DEFAULT nextval('plannedactivitymembers_id_seq'::regclass);


--
-- Name: id; Type: DEFAULT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY plannedactivityreminders ALTER COLUMN id SET DEFAULT nextval('plannedactivityreminders_id_seq'::regclass);


--
-- Name: id; Type: DEFAULT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY users ALTER COLUMN id SET DEFAULT nextval('users_id_seq1'::regclass);


--
-- Data for Name: activities; Type: TABLE DATA; Schema: public; Owner: aeglbot
--

COPY activities (id, name, mode, min_fireteam_size, max_fireteam_size, min_light, min_level) FROM stdin;
1	Vault of Glass	normal	1	6	200	25
2	Vault of Glass	hard	1	6	280	30
3	Crota's End	normal	1	6	200	30
4	Crota's End	hard	1	6	280	33
5	King's Fall	normal	1	6	290	35
6	King's Fall	hard	1	6	320	40
7	Wrath of the Machine	normal	1	6	360	40
8	Wrath of the Machine	hard	1	6	380	40
9	Vanguard	Patrols	1	3	\N	\N
10	Vanguard	any	1	3	\N	\N
11	Crucible	Private Matches	1	12	\N	\N
12	Crucible	Trials of Osiris	3	3	370	40
13	Crucible	Iron Banner	1	6	350	40
14	Crucible	6v6	1	6	\N	\N
15	Crucible	3v3	1	3	\N	\N
16	Crucible	any	1	12	\N	\N
\.


--
-- Name: activities_id_seq; Type: SEQUENCE SET; Schema: public; Owner: aeglbot
--

SELECT pg_catalog.setval('activities_id_seq', 16, true);


--
-- Data for Name: old_users; Type: TABLE DATA; Schema: public; Owner: aeglbot
--

COPY old_users (id, telegram_name, telegram_id, psn_name, email, psn_clan, created_at, updated_at, deleted_at, tokens, pending_activation_code) FROM stdin;
2	berkus	35355684	dozniak	\N	AEGL	2016-10-05 23:50:08.497295	2016-10-05 23:50:08.497295	\N	\N	\N
4	Hunny_Lang	37729935	Hunny_Lang	\N	AEGL	2016-10-05 23:51:34.030308	2016-10-05 23:51:34.030308	\N	\N	\N
5	ergo_guilty	138752347	ergo_guilty	\N	AEGL	2016-10-05 23:53:38.448169	2016-10-05 23:53:38.448169	\N	\N	\N
6	Zestranec	83879240	Zestranec	\N	AEGL	2016-10-05 23:58:09.180055	2016-10-05 23:58:09.180055	\N	\N	\N
8	urbansheep	27829268	urbvnsheep	\N	AEGL	2016-10-06 14:07:14.675947	2016-10-06 14:07:14.675947	\N	\N	\N
9	senweitai	62061037	trancaruas	\N	AEGL	2016-10-06 14:07:14.93151	2016-10-06 14:07:14.93151	\N	\N	\N
10	Phil_sopher	3871444	phil_sopher	\N	AEGL	2016-10-06 14:07:15.006942	2016-10-06 14:07:15.006942	\N	\N	\N
11	Zunou	61044864	zunou_hanasu	\N	AEGL	2016-10-06 14:07:15.334472	2016-10-06 14:07:15.334472	\N	\N	\N
12	tyrann0s	1716773	TRNNS	\N	AEGL	2016-10-06 15:20:31.196214	2016-10-06 15:20:31.196214	\N	\N	\N
13	tapotche	380759	novakche	\N	AEGL	2016-10-06 15:20:31.502289	2016-10-06 15:20:31.502289	\N	\N	\N
14	alexundr	757294	kayouga	\N	AEGL	2016-10-06 15:20:32.244677	2016-10-06 15:20:32.244677	\N	\N	\N
16	kam_aero	11504	aero_kamero	\N	AEGL	2016-10-06 15:28:01.326678	2016-10-06 15:28:01.326678	\N	\N	\N
17	shvrk	71328397	lososnitunca	\N	AEGL	2016-10-06 15:28:26.707688	2016-10-06 15:28:26.707688	\N	\N	\N
20	mbravorus	58450128	TheremHarth	\N	AEGL	2016-10-06 15:46:38.737495	2016-10-06 15:46:38.737495	\N	\N	\N
21	yeong_gu	174904900	oleycon	\N	AEGL	2016-10-06 17:01:34.399122	2016-10-06 17:01:34.399122	\N	\N	\N
22	anpur	95607348	RecursiveGoto	\N	AEGL	2016-10-06 17:02:20.608903	2016-10-06 17:02:20.608903	\N	\N	\N
24	exunitato	79651	exunitato	\N	\N	2016-10-06 17:21:20.601309	2016-10-06 17:21:20.601309	\N	\N	\N
26	cybertiger	78470379	euclid_nikiforov	\N	\N	2016-10-06 18:34:49.666762	2016-10-06 18:34:49.666762	\N	\N	\N
\.


--
-- Data for Name: plannedactivities; Type: TABLE DATA; Schema: public; Owner: aeglbot
--

COPY plannedactivities (id, author_id, activity_id, details, start) FROM stdin;
\.


--
-- Name: plannedactivities_id_seq; Type: SEQUENCE SET; Schema: public; Owner: aeglbot
--

SELECT pg_catalog.setval('plannedactivities_id_seq', 1, false);


--
-- Data for Name: plannedactivitymembers; Type: TABLE DATA; Schema: public; Owner: aeglbot
--

COPY plannedactivitymembers (id, planned_activity_id, user_id, added) FROM stdin;
\.


--
-- Name: plannedactivitymembers_id_seq; Type: SEQUENCE SET; Schema: public; Owner: aeglbot
--

SELECT pg_catalog.setval('plannedactivitymembers_id_seq', 1, false);


--
-- Data for Name: plannedactivityreminders; Type: TABLE DATA; Schema: public; Owner: aeglbot
--

COPY plannedactivityreminders (id, planned_activity_id, user_id, remind) FROM stdin;
\.


--
-- Name: plannedactivityreminders_id_seq; Type: SEQUENCE SET; Schema: public; Owner: aeglbot
--

SELECT pg_catalog.setval('plannedactivityreminders_id_seq', 1, false);


--
-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: aeglbot
--

COPY users (id, telegram_name, telegram_id, psn_name, email, psn_clan, created_at, updated_at, deleted_at, tokens, pending_activation_code) FROM stdin;
2	berkus	35355684	dozniak	\N	AEGL	2016-10-05 23:50:08.497295	2016-10-05 23:50:08.497295	\N	\N	\N
4	Hunny_Lang	37729935	Hunny_Lang	\N	AEGL	2016-10-05 23:51:34.030308	2016-10-05 23:51:34.030308	\N	\N	\N
5	ergo_guilty	138752347	ergo_guilty	\N	AEGL	2016-10-05 23:53:38.448169	2016-10-05 23:53:38.448169	\N	\N	\N
6	Zestranec	83879240	Zestranec	\N	AEGL	2016-10-05 23:58:09.180055	2016-10-05 23:58:09.180055	\N	\N	\N
8	urbansheep	27829268	urbvnsheep	\N	AEGL	2016-10-06 14:07:14.675947	2016-10-06 14:07:14.675947	\N	\N	\N
9	senweitai	62061037	trancaruas	\N	AEGL	2016-10-06 14:07:14.93151	2016-10-06 14:07:14.93151	\N	\N	\N
10	Phil_sopher	3871444	phil_sopher	\N	AEGL	2016-10-06 14:07:15.006942	2016-10-06 14:07:15.006942	\N	\N	\N
11	Zunou	61044864	zunou_hanasu	\N	AEGL	2016-10-06 14:07:15.334472	2016-10-06 14:07:15.334472	\N	\N	\N
12	tyrann0s	1716773	TRNNS	\N	AEGL	2016-10-06 15:20:31.196214	2016-10-06 15:20:31.196214	\N	\N	\N
13	tapotche	380759	novakche	\N	AEGL	2016-10-06 15:20:31.502289	2016-10-06 15:20:31.502289	\N	\N	\N
14	alexundr	757294	kayouga	\N	AEGL	2016-10-06 15:20:32.244677	2016-10-06 15:20:32.244677	\N	\N	\N
16	kam_aero	11504	aero_kamero	\N	AEGL	2016-10-06 15:28:01.326678	2016-10-06 15:28:01.326678	\N	\N	\N
17	shvrk	71328397	lososnitunca	\N	AEGL	2016-10-06 15:28:26.707688	2016-10-06 15:28:26.707688	\N	\N	\N
20	mbravorus	58450128	TheremHarth	\N	AEGL	2016-10-06 15:46:38.737495	2016-10-06 15:46:38.737495	\N	\N	\N
21	yeong_gu	174904900	oleycon	\N	AEGL	2016-10-06 17:01:34.399122	2016-10-06 17:01:34.399122	\N	\N	\N
22	anpur	95607348	RecursiveGoto	\N	AEGL	2016-10-06 17:02:20.608903	2016-10-06 17:02:20.608903	\N	\N	\N
24	exunitato	79651	exunitato	\N	\N	2016-10-06 17:21:20.601309	2016-10-06 17:21:20.601309	\N	\N	\N
26	cybertiger	78470379	euclid_nikiforov	\N	\N	2016-10-06 18:34:49.666762	2016-10-06 18:34:49.666762	\N	\N	\N
\.


--
-- Name: users_id_seq; Type: SEQUENCE SET; Schema: public; Owner: aeglbot
--

SELECT pg_catalog.setval('users_id_seq', 26, true);


--
-- Name: users_id_seq1; Type: SEQUENCE SET; Schema: public; Owner: aeglbot
--

SELECT pg_catalog.setval('users_id_seq1', 2, true);


--
-- Name: activities_pkey; Type: CONSTRAINT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY activities
    ADD CONSTRAINT activities_pkey PRIMARY KEY (id);


--
-- Name: plannedactivities_pkey; Type: CONSTRAINT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY plannedactivities
    ADD CONSTRAINT plannedactivities_pkey PRIMARY KEY (id);


--
-- Name: plannedactivitymembers_pkey; Type: CONSTRAINT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY plannedactivitymembers
    ADD CONSTRAINT plannedactivitymembers_pkey PRIMARY KEY (id);


--
-- Name: plannedactivityreminders_pkey; Type: CONSTRAINT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY plannedactivityreminders
    ADD CONSTRAINT plannedactivityreminders_pkey PRIMARY KEY (id);


--
-- Name: users_pkey; Type: CONSTRAINT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY old_users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: users_pkey1; Type: CONSTRAINT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY users
    ADD CONSTRAINT users_pkey1 PRIMARY KEY (id);


--
-- Name: users_psn_name_key; Type: CONSTRAINT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY old_users
    ADD CONSTRAINT users_psn_name_key UNIQUE (psn_name);


--
-- Name: users_telegram_id_key; Type: CONSTRAINT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY old_users
    ADD CONSTRAINT users_telegram_id_key UNIQUE (telegram_id);


--
-- Name: users_telegram_name_key; Type: CONSTRAINT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY old_users
    ADD CONSTRAINT users_telegram_name_key UNIQUE (telegram_name);


--
-- Name: activities_name; Type: INDEX; Schema: public; Owner: aeglbot
--

CREATE INDEX activities_name ON activities USING btree (name);


--
-- Name: plannedactivitymembers_planned_activity_id_user_id_unique; Type: INDEX; Schema: public; Owner: aeglbot
--

CREATE UNIQUE INDEX plannedactivitymembers_planned_activity_id_user_id_unique ON plannedactivitymembers USING btree (planned_activity_id, user_id);


--
-- Name: users_psn_name_unique; Type: INDEX; Schema: public; Owner: aeglbot
--

CREATE UNIQUE INDEX users_psn_name_unique ON users USING btree (psn_name);


--
-- Name: users_telegram_id_unique; Type: INDEX; Schema: public; Owner: aeglbot
--

CREATE UNIQUE INDEX users_telegram_id_unique ON users USING btree (telegram_id);


--
-- Name: users_telegram_name_unique; Type: INDEX; Schema: public; Owner: aeglbot
--

CREATE UNIQUE INDEX users_telegram_name_unique ON users USING btree (telegram_name);


--
-- Name: plannedactivities_activity_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY plannedactivities
    ADD CONSTRAINT plannedactivities_activity_id_fkey FOREIGN KEY (activity_id) REFERENCES activities(id);


--
-- Name: plannedactivities_author_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY plannedactivities
    ADD CONSTRAINT plannedactivities_author_id_fkey FOREIGN KEY (author_id) REFERENCES users(id);


--
-- Name: plannedactivitymembers_planned_activity_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY plannedactivitymembers
    ADD CONSTRAINT plannedactivitymembers_planned_activity_id_fkey FOREIGN KEY (planned_activity_id) REFERENCES plannedactivities(id);


--
-- Name: plannedactivitymembers_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY plannedactivitymembers
    ADD CONSTRAINT plannedactivitymembers_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id);


--
-- Name: plannedactivityreminders_planned_activity_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY plannedactivityreminders
    ADD CONSTRAINT plannedactivityreminders_planned_activity_id_fkey FOREIGN KEY (planned_activity_id) REFERENCES plannedactivities(id);


--
-- Name: plannedactivityreminders_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: aeglbot
--

ALTER TABLE ONLY plannedactivityreminders
    ADD CONSTRAINT plannedactivityreminders_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id);


--
-- Name: public; Type: ACL; Schema: -; Owner: berkus
--

REVOKE ALL ON SCHEMA public FROM PUBLIC;
REVOKE ALL ON SCHEMA public FROM berkus;
GRANT ALL ON SCHEMA public TO berkus;
GRANT ALL ON SCHEMA public TO PUBLIC;


--
-- PostgreSQL database dump complete
--

