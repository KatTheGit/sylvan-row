# 1. Game design document

All design guidelines for this videogame.

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
    - [4.2.1. Cynewynn](#421-cynewynn)
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
- Primary attack (on short cooldown, generally less than 1s)
- Secondary attack (uses charge, build by damaging opponents or healing allies.)
- Dash (longer cooldown)
- Other
  - Passive abilities
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

Characters must be designed to be playable on a controller.

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
  - Raphaelle tethers to allies within a certain radius.
  - She shoots a medium range piercing projectile, if it damages an opponent, her tethered allies will recieve some health. She recieves half the health her allies recieve.
  - If empowered, her projectile will deal more damage but won't heal allies. If it lands, it will slightly reduce the cooldown on her dash ability.
- Secondary
  - Places down a circular healpool, which periodically heals allies and increases the fire rate of anyone inside.
- Dash
  - A short dash that empowers the next primary shot.
  - Dashing through an ally will heal them. [NOT IMPLEMENTED, might add if her healing output is too low]
- Passives
  - Gains a short temporary speed buff upon getting hit.
  - Relatively fast base movement speed, should be able to keep up with most teammates.
  - Marginally heal tethered allies [NOT IMPLEMENTED, might add if her healing output is too low]

Intended playstyle:
- Has to be able to play offensively so she can play defensively. She can only heal by attacking players.
- Can be very evasive, especially if she lands her empowered shots.

## 4.2. Assassins

### 4.2.1. Cynewynn

- Primary
  - A piercing sword slash.
  - The higher her secondary charge, the faster her attack rate.
- Secondary
  - Teleport back to her previous position, a few seconds in the past.
  - Gain a small amount of health doing so.
- Movement
  - A long dash on a moderately long cooldown.
  - Take reduced damage during the dash.
- Passive
  - Highest movement speed.

Intended playstyle:
- A balanced hit & run. She gains from not retreating.
- She has to be careful. Her dash is on a long cooldown and her secondary reduces her fire rate. Her actions must be planned and she significantly weakens from a hesitant player. Her mobility is her downfall, a smart player might be able to bait out a dash and play around her cooldowns.

## 4.3. Rangers

### 4.3.1. Hernani

- Primary
  - Long range shot, with a relatively low fire rate.
- Secondary
  - Place down a wall, using some secondary charge
- Movement
  - Dash far away, placing down a bear trap. The ability is on a relatively long cooldown.
  - The bear trap arms after half a second, and harms anyone touching it. It lasts until Hernani regains his dash cooldown, or until someone steps on it.
- Passive
  - Has an easier time destroying walls.
  - Slowest movement speed.

Intended playstyle:
- Easy to play, tutorial character.
- Stand back and fire from a distance, while keeping people away with walls and making sure to manage your long cooldown dash adequately.
