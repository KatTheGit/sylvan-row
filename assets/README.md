# 1. Game design document

All design guidelines for this videogame.

- [1. Game design document](#1-game-design-document)
  - [1.1. Basic game description](#11-basic-game-description)
- [2. Basic mechanics](#2-basic-mechanics)
  - [2.1. Controls and Movement](#21-controls-and-movement)
  - [2.2. Stunning and Control](#22-stunning-and-control)
  - [2.3. Combat Visuals and Feedback](#23-combat-visuals-and-feedback)
  - [2.4. Map](#24-map)
- [3. Character Classes](#3-character-classes)
  - [3.1. Assassin / Brawler](#31-assassin--brawler)
  - [3.2. Healer / Tank](#32-healer--tank)
  - [3.3. Ranged / Controller](#33-ranged--controller)
- [4. Characters](#4-characters)
  - [4.1. Support/Tank](#41-supporttank)
    - [4.1.1. Raphaelle](#411-raphaelle)
    - [4.1.2. Wiro \[IDEA\]](#412-wiro-idea)
    - [4.1.3. Alita \[IDEA\]](#413-alita-idea)
  - [4.2. Assassin/Brawler](#42-assassinbrawler)
    - [4.2.1. Cynewynn](#421-cynewynn)
  - [4.3. Ranged/Control](#43-rangedcontrol)
    - [4.3.1. Hernani](#431-hernani)
    - [4.3.2. Elizabeth](#432-elizabeth)
- [5. Gameplay](#5-gameplay)

## 1.1. Basic game description

A multiplayer top-down shooter where players fight with their pick amongst a cast of varied characters. It focuses on the following values:
- Having balanced characters that avoid hard-counter interactions, noob stomping, and unfair abilities.
- Being easy to learn and hard to master
  - Characters are relatively simple yet unique (not many abilities but intricate)
  - Mastering this game means learning how to counter each hero's playstyle
- Having snappy movement (WASD instead of click-to-move), and being controller compatible and **friendly**
- Having hand-drawn but 3D-looking graphics

# 2. Basic mechanics

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

Furthermore, characters are designed to be playable on a controller.

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

## 2.3. Combat Visuals and Feedback

Absolutely everything has to be clear. Everything that is happening on the battlefield.
- One sound effect for every action. A projectile being shot, hitting its target, flying by, someone moving, walls being placed, destroyed, an ability being available, etc...
- As much visual feedback as possible. Show every relationship. Confusing mechanics that are difficult to track must have feedback.
- However, only show partial informtion about opponent ability availability.
- One colour for each projectile type depending on team:
  - Red team, healing: Orange
  - Red team, damage: Red duh
  - Blue team, heal: Green
  - Blue team, damage: Have a guess

## 2.4. Map

Maps contain walls. Walls are a major gameplay element. They can be added (by abilities) or removed (shot down), but they cannot be easily shot or moved through, unless they are temporary.

Maps can contains holes or water. These can be shot through but cannot be moved through. No exceptions.

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
  - She shoots a medium range piercing projectile, if it damages an opponent, her tethered allies will recieve some health. She recieves less health than her allies.
  - If empowered, her projectile will deal more damage but won't heal allies or herself. If it lands, it will reduce the cooldown on her dash ability.
- **Secondary** - Sanctum
  - Places down a tree, which periodically heals allies and increases the fire rate of anyone near it
- **Dash** - Enpowering Leap
  - A short dash that empowers the next primary shot.
- **Passive** - Flight
  - Gains a short temporary speed buff upon getting hit.

Intended playstyle:
- Has to be able to play offensively so she can play defensively. She can only heal by attacking players.
- Can be very evasive, especially if she lands her empowered shots.
- Low DPS but good evasion, she's an underedog character.
- Countered by keeping her away and outranging or by smartly outrunning her


### 4.1.2. Wiro [IDEA]

- **Primary** - Defend
  - A medium range projectile.
  - If it hits an opponent, it adds a stack to **Dash**.
- **Secondary** - Guardian Angel
  - Holds a shield in front of himself, that absorbs damage.
  - If damage is absorbed, secondary charge is decreased accodingly.
  - If his shield is raised, **Passive** no longer applies.
- **Dash** - Combat Medic
  - Dashes forward.
  - Heals any allies that are dashed through. Heals more if he has more stacks.
  - Always has one stack, can go up to three.
- **Passive** - Frontliner
  - Any nearby allies are given a small speed boost, including himself.

### 4.1.3. Alita [IDEA]

- **Primary - Form 1**
  - A long range projectile which can heal allies or damage opponents.
- **Primary - Form 2**
  - A fast short range projectile that either damages an opponent, or heals an ally.
  - If an ally is healed, you are also healed.
- **Secondary** 
  - Alternate between the first and second form.
  - Heal yourself slightly.
  - Charges relatively fast.
  - Form 1 is ranged.
  - Form 2 is an assassin.
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
- Might rework to be an assassin.

## 4.2. Assassin/Brawler

### 4.2.1. Cynewynn

- **Primary** - Righteous Wrath of an Honourable Queen
  - A piercing sword slash, with a relatively fast attack rate.
- **Secondary** - Rectify
  - Passive: Leaves a visual trail behind herself.
  - Teleport back to her previous position, a few seconds in the past, back to the beginning of the trail.
  - Gain a small amount of health doing so.
- **Movement** - Combat Leap
  - A long dash on a moderately long cooldown.
  - Take reduced damage during the dash.
- **Passive** - Dedication
  - The higher her secondary charge, the faster her attack rate.

Intended playstyle:
- A balanced hit & run. She gains from not retreating.
- She has to be careful. Her dash is on a long cooldown and her secondary reduces her fire rate. Her actions must be planned and she significantly weakens from a hesitant player. Her mobility is her downfall, a smart player might be able to bait out a dash and play around her cooldowns.

## 4.3. Ranged/Control

### 4.3.1. Hernani

- **Primary** - Silver Bullet
  - Long range shot, with a relatively low fire rate.
- **Secondary** - Vine Wall
  - Place down a wall, using some secondary charge
- **Movement** - Elusive Retreat
  - Dash far away, placing down a bear trap. The ability is on a relatively long cooldown.
  - The bear trap arms after half a second, and harms anyone touching it. It lasts until Hernani regains his dash cooldown, or until someone steps on it.
- **Passive** - Destructive Weaponry
  - Has an easier time destroying walls.

Intended playstyle:
- Easy to play, tutorial character.
- Stand back and fire from a distance, while keeping people away with walls and making sure to manage your long cooldown dash adequately.

### 4.3.2. Elizabeth

- **Primary** - Knife throw
  - Throw a mid-to-long range knife.
  - At the end of its trajectory, the knife falls on the ground and stays there for a bit.
  - There can only be 3 knives at a time on the ground
- **Secondary** - Ricochet
  - Throw a knife just like **Primary**
  - This kinfe has increased range and slightly higher damage, and can bounce off walls once.
- **Dash** - Recall
  - Dash forward, pulling towards yourself all knives that were on the ground.
  - Returning knives deal small damage to opponents caught in their path.
  - Opponents caught by multiple knives are marginally slowed.

Intended playstyle:
- A character that likes to stand back and control chokeholds.
- Countered by fast characters or widening chokeholds by destroying walls.

# 5. Gameplay

Similar to Battlerite's *current* gamemode.
- 1v1 or 2v2
- Players fight until the last team stands in the arena, with the fight lasting
between 30 seconds and one minute.
- Health can't be fully healed back.
- Best of 3