# 1. Game design document

This document specifies all design rules and choices for this videogame.

- [1. Game design document](#1-game-design-document)
  - [1.1. Basic game description](#11-basic-game-description)
- [2. Base mechanics](#2-base-mechanics)
  - [2.1. Controls and Movement](#21-controls-and-movement)
  - [2.2. Stunning and Control](#22-stunning-and-control)
  - [2.3. Vision and Feedback](#23-vision-and-feedback)
  - [2.4. Map](#24-map)
- [3. Classes](#3-classes)
  - [3.1. Assassin](#31-assassin)
  - [3.2. Healer](#32-healer)
  - [3.3. Ranger](#33-ranger)
- [4. Characters](#4-characters)
  - [4.1. Healers](#41-healers)
    - [4.1.1. Raphaelle](#411-raphaelle)
  - [4.2. Assassins](#42-assassins)
    - [4.2.1. no name yet - IDEA 1 - not yet implemented](#421-no-name-yet---idea-1---not-yet-implemented)
    - [4.2.2. no name yet - IDEA 2 - current implementation](#422-no-name-yet---idea-2---current-implementation)
  - [4.3. Rangers](#43-rangers)
    - [4.3.1. Hernani](#431-hernani)

## 1.1. Basic game description

A multiplayer arena fighting game where players fight with their pick amongst a cast of varied characters. It focuses on the following values:
- Having balanced characters
- Having relatively simple but unique characters (not many abilities but intricate)
- Having snappy movement (WASD instead of click-to-move)
- Being controller compatible and **friendly**
- Being a top-down shooter
- Having hand-drawn but 3D-looking graphics
- Being easy to learn and hard to master

# 2. Base mechanics

There are, independently of the picked character, three types of abilities. These differ by character.
- Primary attack (sort cooldown)
- Secondary attack (uses charge, build up my hitting shots.)
- Dash (longer cooldown)
- Other
  - Passive ability
  - Movement speed
  - Passive ssecondary charge rate

## 2.1. Controls and Movement

The player has five game controls, which are independent of their picked character:
- aim
- move
- shoot
- shoot secondary
- Dash

A player may move freely if unobstructed by the map, aim and shoot freely at all times, and shoot their secondary
once it is charged. These are the player's rights and may not be inhibited by other players and their abilities (i.e. stun, root).

## 2.2. Stunning and Control

The concept of stunning, of inhibiting movement and action, is very unpleasant, and requires to be approached with an
uncomfortable playstyle consisting of extreme cautiousness, disfavouring dynamic combat and boldness.
In light of this, any control character should not:
- Cancel abilities
- Inhibit movement for long periods of time
- Inhibit the use of an ability
- Be annoying (subjective, will depend on player feedback)

However, they can:
- Modify the map (kind of a workaround tbf, to be done with caution)
- Place visible damaging traps.
- Push enemies away slightly.
- Slow them slightly for a short period of time.

## 2.3. Vision and Feedback

Absolutely everything has to be clear. Everything that is happening on the battlefield.
- One sound effect for every action. A projectile being shot, hitting its target, flying by, someone moving, walls being placed, destroyed, an ability being available, etc...
- As much visual feedback as possible. Show every relationship. Confusing mechanics that are difficult to track must have feedback.
- However, only show partial informtion about opponent ability availability.
- Invisibility, be it invisible players, traps or whatnot again requires a ridiculously cautious playstyle that is disfavoured in this game.
- One colour for each projectile type depending on team:
- Red team, healing: Orange
- Red team, damage: Red duh
- Blue team, heal: Green
- Blue team, damage: Have a guess
## 2.4. Map

Maps contain walls. Walls are a major gameplay element. They can be added (by abilities) or removed (shot down), but they cannot be easily shot or moved through, unless they are temporary.

Maps can contains holes or water. These can be shot through but cannot be moved through. No exceptions.

# 3. Classes

These are three classes that create a ternary system. It creates a rock-paper-scissors style countering triangle. Of course all "counters" are soft-counters. There should be no hardcounter matchups. These classes are malleable, and should only be loosely followed.

## 3.1. Assassin

The fastest class. They specialise in high "burst" mobility and have a higher damage output. They have the shortest range. They easily catch up to healers, but may have a bit more trouble with rangers.

Assassins excel at supporting rangers by protecting them from threats like other assassins or healers.

## 3.2. Healer

The second fastest class. Are very evasive and can easily avoid big bullets but not enough to evade assassins. They "counter" rangers' defensive playstyles by simply healing when the ranger plays too defensively. They have the second longest range.

Healers excel at supporting assassins in battle.

## 3.3. Ranger

The slowest but most defensive class. They have a very long range and defensive abilities that make them harder to approach. Assassins have more trouble with them than healers, if the ranger plays carefully.

They excel at helping healers in battle, as they can help defend  them from assassins.

# 4. Characters

## 4.1. Healers

### 4.1.1. Raphaelle

- Primary
  - A lifesteal mid-range shot that sends the health to nearby teammates (or a little to self if no one around).
- Secondary
  - Place down a zone of increased fire rate and health regen
- Movement
  - A short dash that empowers the next shot, which does not heal, but does extra damage. If it lands, it restores dash charge.
- Passive
  - Flight: Gains a short temporary speed buff upon getting hit.
  - Relatively fast base movement speed, should be able to keep up with most teammates.

Intended playstyle:
- Has to be able to play offensively so she can play defensively.
- A somewhat versatile, evasive underdog.
  - If helping the team, deals less damage, less offensive, more defensive
  - If solo, can deal more damage, can be more offensive
  - A good player should merge both playstyles.

## 4.2. Assassins

### 4.2.1. no name yet - IDEA 1 - not yet implemented

- Primary
  - A sword slash. Fire rate is proportional to secondary charge. At 70% or something, it reaches full potential, where it's 5% above standard.
    Below that, it is gradually worse, going as far as 20% below standard.
- Secondary
  - Teleport forward, entering a "dash mode" where you take less damage, using distance-proportional secondary charge. Charge must be at least 50-70%.
- Movement
  - A short flashback.
- Passive
  - no passive

Intended playstyle:
- A balanced hit & run

### 4.2.2. no name yet - IDEA 2 - current implementation

- Primary
  - A sword slash.
- Secondary
  - Flash back to your position 3 seconds ago.
- Movement
  - Dash forward, taking reduced damage during the quick dash.
- Passive
  - no passive

Intended playstyle:
- A balanced hit & run

## 4.3. Rangers

### 4.3.1. Hernani

- Primary
  - Long range shot
- Secondary
  - Place down a wall, using some secondary charge
- Movement
  - Dash away, placing down a landmine/bear trap/whatnot. The landmine arms after a second or two.
- Passive
  - Has an easier time destroying walls

Intended playstyle:
- Easy to play, tutorial character ahh
- Stand back and fire from a distance
