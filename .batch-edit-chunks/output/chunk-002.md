# chunk-002

## Classified Sources

- Line 93: Divine Shards -> `heterogeneous_pair`; the scene contrasts a stabilizing shard with a dangerous high-stakes shard rather than holding one clean cost or reward constant.
- Line 87: The Ssssserpent -> `single_offer`; one deterministic bargain with a visible burden is paired with a neutral leave option.
- Line 66: Battleworn Dummy -> `same_reward_different_costs`; the player calibrates how hard a visible challenge should be for a larger reward in the same reward family.
- Line 60: Heaven's Finest -> `paired_return`; a previous choice returns one battle later as a remembered upgraded card.
- Line 134: Tablet of Truth -> `take_up_to_n`; the player can repeatedly accept the same kind of deck improvement at a visible cumulative cost, then stop.

## Candidate Examples

### timed_window_menu: Battle Lessons

Source: line 66, Battleworn Dummy.
Rationale: This would replace `After Next Battle`; it keeps the shared timing but adds visible battle objectives instead of three unconditional delayed rewards.
Markdown:
```markdown
## Battle Lessons

- After your next battle, if you materialized 3 or more characters, gain 90 essence.
- After your next battle, if one character has 6 or more spark, gain 160 essence.
- After your next battle, if you played 6 or more cards, gain {Ginger Root}.
```

### paired_return: Tempered Return

Source: line 60, Heaven's Finest.
Rationale: This would replace `Buried Card`; it shows the same card-callback shape with a clearer after-battle return and a stronger upgrade choice.
Markdown:
```markdown
## Tempered Return

- First scene:
  - Bury {Aspiring Guardian} until after your next battle.
  - Bury {Scrap Reclaimer} until after your next battle.
  - Bury {Evacuation Enforcer} until after your next battle.
- Later return:
  - Reclaim the buried card with {Golden Transfiguration}.
  - Reclaim the buried card and duplicate it.
```

## Rejected

- Line 93: Divine Shards -> the safe-versus-dangerous artifact bargain is useful, but `Cleanse or Corrupt` already covers this contrast cleanly in Dreamtides vocabulary.
- Line 87: The Ssssserpent -> essence for a Bane is a clean `single_offer`, but it is too close to `Bane Bargain` to improve the current examples.
- Line 134: Tablet of Truth -> repeatable transfiguration for a burden overlaps with `Layered Cleansing`, `Widen The Rewrite`, and `Chase The Upgrade`; a new example would mostly add another scaling upgrade menu.
