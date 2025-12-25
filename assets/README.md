# 1. Game Design Document

This document specifies guidelines to aid with the development of this game from a design perspective. It is split in the following sections:

- [1. Game Design Document](#1-game-design-document)
  - [1.1. Game Description](#11-game-description)
- [2. Basic Mechanics](#2-basic-mechanics)
  - [2.1. Abilities and Controls](#21-abilities-and-controls)
  - [2.2. Crowd Control and Punishing](#22-crowd-control-and-punishing)
  - [2.3. Combat Visuals and Feedback](#23-combat-visuals-and-feedback)
  - [2.4. Map](#24-map)
- [3. Character Classes](#3-character-classes)
  - [3.1. Assassin / Brawler](#31-assassin--brawler)
  - [3.2. Healer / Tank](#32-healer--tank)
  - [3.3. Ranged / Controller](#33-ranged--controller)
- [4. Characters](#4-characters)
  - [4.1. Support/Tank](#41-supporttank)
    - [4.1.1. Raphaelle](#411-raphaelle)
    - [4.1.2. Wiro](#412-wiro)
    - [4.1.3. Alita \[not implemented\]](#413-alita-not-implemented)
  - [4.2. Assassin/Brawler](#42-assassinbrawler)
    - [4.2.1. Cynewynn](#421-cynewynn)
    - [4.2.2. Temerity](#422-temerity)
  - [4.3. Ranged/Control](#43-rangedcontrol)
    - [4.3.1. Hernani](#431-hernani)
    - [4.3.2. Josey](#432-josey)
- [5. Gameplay](#5-gameplay)
  - [5.1. Gamemode](#51-gamemode)
  - [5.2. Ranked System](#52-ranked-system)
- [6. Lore](#6-lore)
  - [6.1. Setting](#61-setting)
  - [6.2. Characters](#62-characters)

## 1.1. Game Description

A multiplayer top-down shooter where players fight with their pick amongst a cast of varied characters. It focuses on the following values:
- Having balanced characters that avoid (to a certain extent) hard-counter interactions, noob stomping, and unfair abilities.
- Being easy to learn and hard to master
  - Characters are relatively simple yet unique (not many abilities but intricate)
  - Mastering this game means learning how to counter each hero's playstyle
- Having snappy movement (WASD instead of click-to-move), and being controller friendly.
- Having hand-drawn but 3D-looking graphics

# 2. Basic Mechanics

## 2.1. Abilities and Controls

Each character has 3 active abilities and optionally passive abilities. This creates a simple structure, even if the abilities themselves can be complicated.
- **Primary** - An ability on a short cooldown, and is usually the primary means of damaging opponents.
- **Secondary** -  An ablity that needs to be charged. Charge can be built up either passively, by damaging opponents with certain abilities, or by healing allies with certain abilities. It goes up to 100. Characters usually require 100 charge to use their secondary, but there are many exceptions.
- **Movement** - An ability on a longer cooldown and provides mobility, often through the form of a dash. Can also have other effects.
- **Passive** - a character can have many of these, or none at all. They are either always active, or can be triggered by an event.
- Characters also have independent movement speeds.

This results in only 5 game controls:
- move
- aim
- shoot primary
- shoot secondary
- dash

This makes the game controller-friendly and simple to learn.

## 2.2. Crowd Control and Punishing

A player may move freely if unobstructed by the map, aim and shoot freely at all times, and shoot their secondary
once it is charged. These are the player's rights and may not be inhibited by other players and their abilities (i.e. stun, root).

The concept of stunning, of inhibiting movement and actions, is unpleasant, and requires to be approached with an
uncomfortable playstyle consisting of extreme cautiousness, disfavouring dynamic combat and boldness, and excessively punishes beginners, a fragile playerbase.
In light of this, any control character should not:
- Cancel/inhibit abilities
- Inhibit movement for long periods of time

"Light" crowd control like small slows are tolerated. While CC only exists to an extent, this doesn't mean players can't be punished; it's just done in other ways (mostly through damage). Bad positioning might for example be punished by taking useless damage.

## 2.3. Combat Visuals and Feedback

Absolutely everything has to be clear. Everything that is happening on the battlefield.
- Sounds have different identities depending on the type of action they represent. Hitting an opponent might make a "pop"-like sound, while firing your primary might make a "whoosh"-like sound, and taking damage might make a "ping"-like sound.
- As much visual feedback as possible. Show every relationship. Confusing mechanics that are difficult to track must have feedback.
- One colour for each projectile type depending on team:
  - Red team, healing: Orange
  - Blue team, heal: Green
  - Red team, damage: Red duh
  - Blue team, damage: Have a guess

If a character is mechanically simple, compensate visually.

## 2.4. Map

Maps contain walls. Walls are a major gameplay element. They can be added (by abilities) or removed (shot down), but they cannot be easily shot or moved through, unless they are temporary.

Maps can contains holes or water. These can be shot through but cannot be moved through. No exceptions.

Maps should also have a relatively open center, to force players to fight eachother instead of hiding behind walls.

The objective at the center of the map should also help with wall camping.

# 3. Character Classes

There are three classes that create a ternary system. It makes for a rock-paper-scissors style countering triangle. Of course all "counters" are soft-counters. There should be no hardcounter matchups. These classes are malleable, and should only be loosely followed.

## 3.1. Assassin / Brawler

The fastest class. They specialise in high "burst" mobility and have a higher damage output. They have the shortest range. They easily catch up to healers, but may have a bit more trouble with ranged characters.

Assassins excel at supporting ranged characters by protecting them from threats like other assassins or healers.

## 3.2. Healer / Tank

The second fastest class. Are very evasive and can easily avoid big bullets but not enough to evade assassins. They "counter" rangers' defensive playstyles by healing during downtime. They have the second longest range.

Healers excel at supporting assassins in battle.

## 3.3. Ranged / Controller

The slowest but most defensive class. They have a very long range and defensive abilities that make them harder to approach. Assassins have more trouble with them than healers, if the ranger plays carefully.

They excel at helping healers in battle, as they can help defend them from assassins.

# 4. Characters

## 4.1. Support/Tank

### 4.1.1. Raphaelle

- **Primary** - Kindness in Blood
  - Raphaelle tethers to allies within a certain radius.
  - She shoots a medium range piercing projectile, if it damages an opponent, her tethered allies will recieve some health. She recieves health, but less.
  - If empowered, her projectile will deal more damage but won't heal allies or herself. If it lands, it will reduce the cooldown of **Dash**.
- **Secondary** - Sanctum
  - Places down a healpool, which periodically heals allies and increases the fire rate of anyone near it
- **Dash** - Enpowering Leap
  - A short dash that empowers the next **Primary**.
- **Passive** - Flight
  - Gains a short temporary speed buff upon getting hit.

Intended playstyle:
- Has to be able to play offensively so she can play defensively. She can only heal by attacking players.
- Can be very evasive, especially if she lands her empowered shots.
- Low DPS but good evasion, she's an underedog character.
- Countered by keeping her away and outranging or by smartly outrunning her

### 4.1.2. Wiro

- **Primary** - Buck-Shot
  - A shotgun with a tight spread. Does more damage up close.
  - If he damages an opponent, it empowers the next **Dash**.
- **Secondary** - Guardian
  - Holds a shield in front of himself that absorbs damage. If damage is absorbed, secondary charge is decreased.
  - Raising his shield disables **Primary** and **Passive**.
  - After raising his shield, he cannot raise it again for a few seconds.
- **Dash** - Intervene
  - A long-ish dash on a medium cooldown.
  - If empowered, heals the allies he passes through and damages the opponents he passes through.
- **Passive** - Inspire
  - Provides a small movement buff to nearby allies (including himself)

Intended playstyle:
- Protect allies with shield or empower them.
- Punish people who get close to you -> applies pressure
- Dash empowerment and shield depletion forces you to fight instead of shieldbot + healbot
- Counters:
  - Outranged
  - Ganked
  - Ignored

### 4.1.3. Alita [not implemented]

- **Primary - Form 1**
  - A long range projectile which can heal allies or damage opponents.
- **Primary - Form 2**
  - A fast short range projectile that either damages an opponent, or heals an ally.
  - If an ally is healed, you are also healed.
- **Secondary**
  - Alternate between the first and second form.
  - Slightly heal yourself and your allies.
- **Dash - Form 1**
  - A medium dash on a medium cooldown.
- **Dash - Form 2**
  - A long dash on a long cooldown.
- **Passive - Form 2**
  - Marginally higher movement speed

Intended playstyle:
- The health gained from the secondary is an incentive to change forms.
- You must therefore adapt to two playstyles.
- Countered by punishing bad positioning and forcing out **Secondary**.

Note
- Might rework to be an assassin

## 4.2. Assassin/Brawler

### 4.2.1. Cynewynn

- **Primary** - Wrath
  - A sword slash, with a relatively fast attack rate.
- **Secondary** - Rectify
  - Leaves a trail behind herself.
  - Teleport back to her previous position, a few seconds in the past, back to the beginning of the trail.
  - Gain a small amount of health doing so.
- **Dash** - Combat Leap
  - A long dash on a moderately long cooldown.
  - Take reduced damage during the dash.
- **Passive** - Dedication
  - The higher her secondary charge, the faster her attack rate.

Intended playstyle:
- A balanced hit & run. She gains from not retreating.
- She has to be careful. Her dash is on a long cooldown and her secondary reduces her fire rate. Her actions must be planned and she significantly weakens from a hesitant player. Her mobility is her downfall, a smart player might be able to bait out a dash and play around her cooldowns.

### 4.2.2. Temerity

- **Primary** - The Subtle Art of a Rocket Launcher
  - A three-hit combo:
    - A short range attack
    - A mid-range attack
    - A difficult to hit long-range attack
  - If you hit an opponent, you switch to the next attack. Hitting an opponent with the last attack doesn't switch attacks. Missing any attack resets it to the first one.
- **Secondary** - Rocket Jump
  - Launch a rocket under yourself that damages opponents in an AOE and boosts you forward.
- **Dash** - Hasty Dispatch
  - Holding **Dash** lets you wallride, letting you move quickly alongside walls.
  - You carry momentum from wallriding after after releasing.
- **Passive** - Save the City
  - Heal damaged walls around you.

Intended playstyle:
- zoop in and out weeee
- Reward people for getting close by giving them two safe shots
- ROCKET LAUNCHER RAHHHHHH
- Combo heavy

## 4.3. Ranged/Control

### 4.3.1. Hernani

- **Primary** - Silver Bullet
  - Long range shot, with a relatively low fire rate.
- **Secondary** - Vine Wall
  - Place down a wall, using some secondary charge
- **Dash** - Elusive Retreat
  - Dash away, placing down a bear trap. The ability is on a relatively long cooldown.
  - The bear trap arms after half a second, and harms anyone touching it. It lasts until Hernani regains his dash cooldown, or until someone steps on it.
- **Passive** - Destructive Weaponry
  - Has an easier time destroying walls.

Intended playstyle:
- Easy to play, tutorial character.
- Stand back and fire from a distance, while keeping people away with walls and making sure to manage your long cooldown dash adequately.

### 4.3.2. Josey

- **Primary** - Cunning Knives
  - Throw a mid-range knife tha can ricochet off walls once.
  - If the knife bounces, its range increases.
  - At the end of its trajectory, the knife falls on the ground and stays there for a bit. There can only be 2 knives at a time on the ground.
- **Secondary** - Thingymajig
  - Create a walking turret that moves in the direction it was cast.
  - It pinballs around the map and shoots nearby opponents.
- **Dash** - Recall
  - Dash forward, pulling towards yourself all knives that were on the ground.
  - Returning knives deal small damage to opponents caught in their path.
  - Opponents caught by multiple knives are marginally slowed.

Intended playstyle:
- Controls chokeholds and can force people around obstacles.
- Countered by getting rid of walls and chasing.

# 5. Gameplay

The format that surrounds the fighting itself.

## 5.1. Gamemode
- Format
  - 1v1 or 2v2
  - Both teams fight until only one team remains.
  - Best of 3
  - 1min intended length for rounds 
- Gameplay
  - There is little healing overall
  - Every 30s or so, an orb spawns in the middle of the map.
    - Whoever gets the last hit on the orb gives their whole team a bit of health.
    - Shooting the orb knocks it back a little.

## 5.2. Ranked System

The rank of a player is determined entirely through their winrate, since no skill-based or ranked-only matchmaking exists (yet).

Each rank has 3 divisions (III to I). Going up a division requires 100 rp. In the last division of a rank, the rp gain/loss is equal to that of the rank above.

| num | RP | name | winrate | win | loss
|---|---|---|---|---|---|
1 | 0 | Bronze | 0% | 50 | 0
2 | 300 | Silver | 25% | 60 | 20
3 | 600 | Gold | 50% | 50 | 50
4 | 900 | ??? | 60% | 34 | 50
5 | 1200 | ??? | 70% | 21 | 50
6 | 1500 | Mystic (n) | 80% | 13-0 ($\frac{100}{0.2 * (RP - 1500) + 0.13}$) | 50

Mystic rank has your leaderboard position appended, and doesn't have any divisions.

You cannot derank 4 divisions (1.33 ranks) below your highest rank. Ranks are reset to 0 every month.

# 6. Lore

The lore serves as guidelines for creating characters. In-game at least, the tidbits of lore should try to be funny and unserious. This is not the case here because I need to condense the world into this document, so do be kind enough to forgive my unpolished and description-heavy writing.

## 6.1. Setting

The world of Sylvan Row is that of a huge forest, populated by little villages and only one remarkable landmark, the City, being the only urban setting of the world. Most animal species of the world are humanoid, but not too much thought should be put into things like evolution and how they came to be or whatnot.

The largest of the villages in the forest, and arguably the second most important landmark, is the Sanctum. It is home to a devoted group of religious animals, known as Angels. Their religion is widespread and accepted as truth (because it is). The religion consists of taking care of the Forest, which is a manifestation of God, in return of its shelter. This is done by the three ranks of Angels:
- Seraphim are capable of understanding how God manifests herself in the flora. They are sometimes seen as insane, but most of the time are not seen at all, and have specialised healing magic which can appease God.
- Archangels are the only ones who can understand the Seraphim. One of their roles is to translate the findings of the latter and issue orders regarding whatever mission may help God. However they are primarily powerful healers who specialise in healing other animals. The kind of healing they specialise in differs from each Archangel. As a symbol of trust, they are often veiled.
- Guardian Angels may sometimes take orders from Archangels but their primary role is to protect people, whatever that may mean. Some are very proficient in combat, others are diplomats, and some are jacks-of-all-trades. They all share a unique connection to God and a strong sense of morality, much more than any other rank of Angel.

The City is a civilisation of animals that decided to live without the assistance of the forest, but not necessarily out of a lack of faith. Certain Angels endorse this landmark while others dislike it, but God seems to have no issue with it. The City's history is troubled, but nowadays is a very pleasant place to live in. While electricity does not exist in this steampunk-retrofuturist world, it remains relatively technologically advanced, with an enourmous tramway network. As it focuses on the well-being of citizens primarily, it is full of gardens and the architecture is very deliberate, despite its high density. It is technologically on par with us, but by far surpasses us socially and in its economic structure.

The city's primary power source is nuclear power. Water is heated and pressurised, to be sent off to a grid of steam pipes. Most devices are steam-operated, and "batteries" are just canisters. The City's political-economic structure makes the existence of monopolies nearly impossible, however one prevails: the only toilet company in existence. As it turns out, toiletsmith is not the most popular of aspirations, and nobody seems to have noticed the megacorporation's reign.

Both the City and the forest are home to a group of anarchists. Some of them lack faith (trust in God), and see her mistakes as a liability, and despise the fact that the city tolerates this religion instead of being a "true refuge", while others are simply insane. The forest is victim of their ways, and defends itself without fighting back too much. The City is isn't fond of them either, but thanks to the advanced state of psychiatrical medicine, is capable of helping a few of them, and is generally a bit more sympathetic.

The forest is also home to Eternals, a group of creatures of unknown origin to God herself, who have roamed for as long as any scripture remembers.

## 6.2. Characters

All names are subject to change.

Raphaelle is an Archangel and a combat medic, but over her many battles had come to enjoy the "combat" part of medic, learning fighting skills and tying them with her healing magic, creating a hybrid fighting style. Some Angels are concerned she might be enjoying combat a bit more than a sworn pacifist should, but she remains a valuable asset.

Hernani is a bandit and a member of the anarchists, convinced of being shunned by society. He is a very troubled person, at conflict with himself much more than he is with anybody else. His paranoia led him to a very cautious fighting style, which he has nevertheless mastered, making him a feared musketeer. His musket appears to have been made by Josey.

Cynewynn is the current ruler of the City. She is the one who has brought to fruition all the changes that made the city into its current utopia - but at a cost. To seize the throne, she had to kill many people in power, including her husband. This bloodshed was not reflected in her ruling and was a purely pragmatic act. Nowadays she feels gutting remorse for what she did, completely ignoring the new world she created. She's more proficient than anyone could even dream of being in the arcane art of manipulating time, but no matter how strong her magic may ever get, she cannot undo what she did. She refuses to recognise it, at the detriment of her sanity, entirely convinced this problem is still within her reach, after so, so many years. Regardless of this, she has somehow remained a pleasant ruler, and has been slowly dissipating her power to create a more open government - one where experts of various fields may one day help the city directly.

Temerity used to be part of the City's police force, but after the changes brought by Cynewynn's reign she was fired due to her methods being too "old school". In fairness, her habit of solving every issue with rocket launchers and her proficiency at crashing police-issued jets was already somewhat frowned upon even in the "old days". However she still performs her duties as a law enforcement officer by stealing equipment from the headquarters and enforcing whatever the current legal code may be, despite the entire task force dedicated to her.

Wiro is a Guardian Angel weilding a gigantic shield and shotgun, dedicating his life to protect others by whatever means necessary (more often than not, the means being a gigantic shield). His dedication does concern his close firends; Sometimes, it's almost like he *wants* to sacrifice himself. Nevertheless, he is incredibly caring towards anyone, regardless of who they may be. He is among the Angels who like the City.

Josey was born with the power to partially intuit coincidence, and manipulate it into deterministic outcomes. In his early years he used this power to make a living through his holistic assasination agency, whose main selling point was the lack of evidence left behind by his "coincidental" methods. As the world changed around him, he took the opportunity to stop his evil trade and become an engineer instead. His intuition made of him a great mechanic, and if he ever had to fight, he'd do it semi-reluctantly, relying on his contraptions to do his bidding. Him and Hernani are rivals, and constantly try to assassinate eachother, but there's something about the way they do it... the hatred isn't sincere, but something else is...

One of the eternals is an odd ornithoid creature which experiences constant, insufferable pain, and exists for the sole purpose of being in pain. Anyone who crosses their path is struck with a sickening feeling of empathy, scarring even the hardest of souls and most weathered of warriors. Nobody leaves the encounter unchanged. The eternal's reaction to pain varies, often being passive and defeated, but in other times erratic and agitated.