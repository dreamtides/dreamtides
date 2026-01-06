e = <color=#00838F>{$e}●</color>
mode1-cost = <color=#00838F>{$mode1-cost}●</color>
mode2-cost = <color=#00838F>{$mode2-cost}●</color>
points = <color=#F57F17>{$points}⍟</color>

energy-symbol = <color=#00838F>●</color>

maximum-energy = {$max} maximum {energy-symbol}

-trigger = ▸ <b>{$trigger}:</b>
Materialized = {-trigger(trigger: "Materialized")}
Judgment = {-trigger(trigger: "Judgment")}
Dissolved = {-trigger(trigger: "Dissolved")}
MaterializedJudgment = {-trigger(trigger: "Materialized, Judgment")}
MaterializedDissolved = {-trigger(trigger: "Materialized, Dissolved")}
JudgmentPhaseName = <b>Judgment</b>

-keyword = <color=#AA00FF>{$k}</color>

dissolve = {-keyword(k: "dissolve")}
dissolved = {-keyword(k: "dissolved")}
Dissolve = {-keyword(k: "Dissolve")}
banish = {-keyword(k: "banish")}
Banish = {-keyword(k: "Banish")}
banished = {-keyword(k: "banished")}
discover = {-keyword(k: "discover")}
Discover = {-keyword(k: "Discover")}
reclaim = {-keyword(k: "reclaim")}
Reclaim = {-keyword(k: "Reclaim")}
materialize = {-keyword(k: "materialize")}
Materialize = {-keyword(k: "Materialize")}
prevent = {-keyword(k: "prevent")}
Prevent = {-keyword(k: "Prevent")}
kindle = {-keyword(k: "kindle")} {$k}
Kindle = {-keyword(k: "Kindle")} {$k}
foresee = {-keyword(k: "foresee")} {$foresee}
Foresee = {-keyword(k: "Foresee")} {$foresee}
fast = <b>↯fast</b>
Fast = <b>↯Fast</b>

reclaim-for-cost = {-keyword(k: "reclaim")} <color=#00838F>{$reclaim}●</color>
ReclaimForCost = {-keyword(k: "Reclaim")} <color=#00838F>{$reclaim}●</color>

ChooseOne = <b>Choose One:</b>
bullet = •

cards =
  {
    $cards ->
      [one] a card
      *[other] { $cards } cards
  }

discards =
  {
    $discards ->
      [one] a card
      *[other] { $discards } cards
  }

top-n-cards =
  {
    $to-void ->
      [one] top card
      *[other] top { $to-void } cards
  }

cards-numeral =
  {
    $cards ->
      [one] { $cards } card
      *[other] { $cards } cards
  }

s = { $s }

spark = spark

count = { $count }

count-allies =
  {
    $allies ->
      [one] an ally
      *[other] { $allies } allies
  }

count-allied-subtype =
  {
    $allies ->
      [one] an allied {subtype}
      *[other] { $allies } allied {plural-subtype}
  }

-figment = <color=#F57F17><b><u>{$f} Figment</u></color></b>

-figments-plural = <color=#F57F17><b><u>{$f} Figments</u></color></b>

figments =
  {
    $figment ->
      [celestial] {-figments-plural(f: "Celestial")}
      [halcyon] {-figments-plural(f: "Halcyon")}
      [radiant] {-figments-plural(f: "Radiant")}
      *[other] Error: Unknown 'figment' for type: { $figment }
  }

a-figment =
  {
    $figment ->
      [celestial] a {-figment(f: "Celestial")}
      [halcyon] a {-figment(f: "Halcyon")}
      [radiant] a {-figment(f: "Radiant")}
      *[other] Error: Unknown 'a-figment' for type: { $figment }
  }

n-figments =
  {
    $number ->
      [one] {a-figment}
      *[other] { text-number } {figments}
  }

-type = <color=#2E7D32><b>{$value}</b></color>

a-subtype =
  {
    $subtype ->
      [ancient] an {-type(value: "ancient")}
      [child] a {-type(value: "child")}
      [detective] a {-type(value: "detective")}
      [enigma] an {-type(value: "enigma")}
      [explorer] an {-type(value: "explorer")}
      [hacker] a {-type(value: "hacker")}
      [mage] a {-type(value: "mage")}
      [monster] a {-type(value: "monster")}
      [musician] a {-type(value: "musician")}
      [outsider] an {-type(value: "outsider")}
      [renegade] a {-type(value: "renegade")}
      [spirit-animal] a {-type(value: "spirit animal")}
      [super] a {-type(value: "super")}
      [survivor] a {-type(value: "survivor")}
      [synth] a {-type(value: "synth")}
      [tinkerer] a {-type(value: "tinkerer")}
      [trooper] a {-type(value: "trooper")}
      [visionary] a {-type(value: "visionary")}
      [visitor] a {-type(value: "visitor")}
      [warrior] a {-type(value: "warrior")}
      *[other] Error: Unknown 'a-subtype' for type: { $subtype }
  }

ASubtype =
  {
    $subtype ->
      [ancient] An {-type(value: "ancient")}
      [child] An {-type(value: "child")}
      [detective] An {-type(value: "detective")}
      [enigma] An {-type(value: "enigma")}
      [explorer] An {-type(value: "explorer")}
      [hacker] An {-type(value: "hacker")}
      [mage] An {-type(value: "mage")}
      [monster] An {-type(value: "monster")}
      [musician] An {-type(value: "musician")}
      [outsider] An {-type(value: "outsider")}
      [renegade] An {-type(value: "renegade")}
      [spirit-animal] An {-type(value: "spirit animal")}
      [super] An {-type(value: "super")}
      [survivor] An {-type(value: "survivor")}
      [synth] An {-type(value: "synth")}
      [tinkerer] An {-type(value: "tinkerer")}
      [trooper] An {-type(value: "trooper")}
      [visionary] An {-type(value: "visionary")}
      [visitor] An {-type(value: "visitor")}
      [warrior] An {-type(value: "warrior")}
      *[other] Error: Unknown 'ASubtype' for type: { $subtype }
  }

subtype =
  {
    $subtype ->
      [ancient] {-type(value: "ancient")}
      [child] {-type(value: "child")}
      [detective] {-type(value: "detective")}
      [enigma] {-type(value: "enigma")}
      [explorer] {-type(value: "explorer")}
      [hacker] {-type(value: "hacker")}
      [mage] {-type(value: "mage")}
      [monster] {-type(value: "monster")}
      [musician] {-type(value: "musician")}
      [outsider] {-type(value: "outsider")}
      [renegade] {-type(value: "renegade")}
      [spirit-animal] {-type(value: "spirit animal")}
      [super] {-type(value: "super")}
      [survivor] {-type(value: "survivor")}
      [synth] {-type(value: "synth")}
      [tinkerer] {-type(value: "tinkerer")}
      [trooper] {-type(value: "trooper")}
      [visionary] {-type(value: "visionary")}
      [visitor] {-type(value: "visitor")}
      [warrior] {-type(value: "warrior")}
      *[other] Error: Unknown 'type' for type: { $subtype }
  }

plural-subtype =
  {
    $subtype ->
      [ancient] {-type(value: "ancients")}
      [child] {-type(value: "children")}
      [detective] {-type(value: "detectives")}
      [enigma] {-type(value: "enigmas")}
      [explorer] {-type(value: "explorers")}
      [hacker] {-type(value: "hackers")}
      [mage] {-type(value: "mages")}
      [monster] {-type(value: "monsters")}
      [musician] {-type(value: "musicians")}
      [outsider] {-type(value: "outsiders")}
      [renegade] {-type(value: "renegades")}
      [spirit-animal] {-type(value: "spirit animals")}
      [super] {-type(value: "supers")}
      [survivor] {-type(value: "survivors")}
      [synth] {-type(value: "synths")}
      [tinker] {-type(value: "tinkerers")}
      [trooper] {-type(value: "troopers")}
      [visionary] {-type(value: "visionaries")}
      [visitor] {-type(value: "visitors")}
      [warrior] {-type(value: "warriors")}
      *[other] Error: Unknown 'plural-type' for type: { $subtype }
  }

text-number =
  {
    $number ->
      [1] one
      [2] two
      [3] three
      [4] four
      [5] five
      [6] six
      [7] seven
      [8] eight
      [9] nine
      *[other] { $number }
  }

this-turn-times =
  {
    $number ->
      [1] this turn
      [2] this turn twice
      *[other] this turn {text-number} times
  }

MultiplyBy =
  {
    $number ->
      [2] Double
      [3] Triple
      [4] Quadruple
      [5] Quintuple
      *[other] Multiply by { $number }
  }

copies =
  {
    $number ->
      [one] a copy
      *[other] { text-number } copies
  }

n-random-characters =
  {
    $number ->
      [1] a random character
      *[other] { text-number } random characters
  }

up-to-n-events  =
  {
    $number ->
      [1] an event
      [2] one or two events
      *[other] up to { $number } events
  }

up-to-n-allies  =
  {
    $number ->
      [1] an ally
      [2] one or two allies
      *[other] up to { $number } allies
  }

it-or-them =
  {
    $number ->
      [1] it
      *[other] them
  }