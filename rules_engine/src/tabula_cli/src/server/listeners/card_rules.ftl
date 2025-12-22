e = <color=#00838F>{$e}●</color>

energy-symbol = <color=#00838F>●</color>

-trigger = ▸ <b>{$trigger}:</b>
Materialized = {-trigger(trigger: "Materialized")}
Judgment = {-trigger(trigger: "Judgment")}
Dissolved = {-trigger(trigger: "Dissolved")}
Materialized-Judgment = {-trigger(trigger: "Materialized, Judgment")}
MaterializedDissolved = {-trigger(trigger: "Materialized, Dissolved")}


-keyword = <color=#AA00FF>{$k}</color>

dissolve = {-keyword(k: "dissolve")}
dissolved = {-keyword(k: "dissolved")}
Dissolve = {-keyword(k: "Dissolve")}
banish = {-keyword(k: "banish")}
Banish = {-keyword(k: "Banish")}
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
kindle-k2 = {-keyword(k: "kindle")} {$k2}
Kindle-k2 = {-keyword(k: "Kindle")} {$k2}
fast = ↯Fast

cards =
  {
    $cards ->
      [one] a card
      *[other] { $cards } cards
  }

cards-v2 =
  {
    $cards-v2 ->
      [one] a card
      *[other] { $cards-v2 } cards
  }

top-n-cards =
  {
    $cards ->
      [one] top card
      *[other] top { $cards } cards
  }

s = { $s }

spark = spark

count-allies =
  {
    $allies ->
      [one] an ally
      *[other] { $allies } allies
  }

figment = <color=#F57F17><b><u>{$figment} Figment</u></color></b>

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
      *[other] Error: Unknown 'a-type' for type: { $subtype }
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
      [survivors] {-type(value: "survivors")}
      [synths] {-type(value: "synths")}
      [tinkerers] {-type(value: "tinkerers")}
      [troopers] {-type(value: "troopers")}
      [visionaries] {-type(value: "visionaries")}
      [visitors] {-type(value: "visitors")}
      [warrior] {-type(value: "warriors")}
      *[other] Error: Unknown 'plural-type' for type: { $subtype }
  }
