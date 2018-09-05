INSERT INTO activities (id, name, mode, min_fireteam_size, max_fireteam_size, min_light, min_level)
VALUES
(41, 'Vanguard', 'Escalation Protocol',          1,   9,   350, null),
(42, 'Leviathan, Spire of Stars', 'normal',      6,   6,   370, 30),
(43, 'Leviathan, Spire of Stars', 'prestige',    6,   6,   385, 30),
(44, 'King''s Fall',              'weekly',      6,   6,   390, 40),
(45, 'Crota''s End',              'weekly',      6,   6,   390, 40),
(46, 'Vault of Glass',            'weekly',      6,   6,   390, 40),
(47, 'Wrath of the Machine',      'weekly',      6,   6,   390, 40),
(48, 'Last Wish',                 'normal',      6,   6,   450, 40),
(49, 'Last Wish',                 'prestige',    6,   6,   500, 40),
(50, 'Gambit',                    'pve/pvp',     1,   4,   400, 30);

INSERT INTO activityshortcuts (id, name, game, link)
VALUES
(46, 'escal8', 'Destiny 2', 41),
(47, 'spiren', 'Destiny 2', 42),
(48, 'spirep', 'Destiny 2', 43),
(49, 'kfw'   , 'Destiny'  , 44),
(50, 'crw'   , 'Destiny'  , 45),
(51, 'vogw'  , 'Destiny'  , 46),
(52, 'wotmw' , 'Destiny'  , 47),
(53, 'lastwn', 'Destiny 2', 48),
(54, 'lastwp', 'Destiny 2', 49),
(55, 'gambit', 'Destiny 2', 50);
