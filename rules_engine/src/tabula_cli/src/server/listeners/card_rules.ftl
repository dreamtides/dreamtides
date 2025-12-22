e = <color=#00838F>{$e}●</color>

-trigger = ▸ <b>{$trigger}:</b>
Materialized = {-trigger(trigger: "Materialized")}
Judgment = {-trigger(trigger: "Judgment")}
Materialized-Judgment = {-trigger(trigger: "Materialized, Judgment")}

-keyword = <color=#AA00FF><b>{$k}</b></color>

Dissolve = {-keyword(k: "Dissolve")}
Banish = {-keyword(k: "Banish")}
Disable = {-keyword(k: "Disable")}
Discover = {-keyword(k: "Discover")}
Reclaim = {-keyword(k: "Reclaim")}
Play = {-keyword(k: "Play")}
Materialize = {-keyword(k: "Materialize")}
fast = {-keyword(k: "↯")}

cards =
  {
    $cards ->
      [one] a card
      *[other] { $cards } cards
  }

s = { $s }

spark = spark

count-allies =
  {
    $allies ->
      [one] an ally
      *[other] { $allies } allies
  }

figment = <color=#F57F17><b><u>{$figment}</u></color></b> Figment

-begin-type = <color=#2E7D32><b>
-end-type = </b></color>

a-subtype =
  {
    $type ->
      [ancient] an {-begin-type}ancient{-end-type}
      [child] a {-begin-type}child{-end-type}
      [detective] a {-begin-type}detective{-end-type}
      [enigma] an {-begin-type}enigma{-end-type}
      [explorer] an {-begin-type}explorer{-end-type}
      [hacker] a {-begin-type}hacker{-end-type}
      [mage] a {-begin-type}mage{-end-type}
      [monster] a {-begin-type}monster{-end-type}
      [musician] a {-begin-type}musician{-end-type}
      [outsider] an {-begin-type}outsider{-end-type}
      [renegade] a {-begin-type}renegade{-end-type}
      [spirit-animal] a {-begin-type}spirit animal{-end-type}
      [super] a {-begin-type}super{-end-type}
      [survivor] a {-begin-type}survivor{-end-type}
      [synth] a {-begin-type}synth{-end-type}
      [tinkerer] a {-begin-type}tinkerer{-end-type}
      [trooper] a {-begin-type}trooper{-end-type}
      [visionary] a {-begin-type}visionary{-end-type}
      [visitor] a {-begin-type}visitor{-end-type}
      [warrior] a {-begin-type}warrior{-end-type}
      *[other] Error: Unknown 'a-type' for type: { $type }
  }

subtype =
  {
    $type ->
      [ancient] {-begin-type}ancient{-end-type}
      [child] {-begin-type}child{-end-type}
      [detective] {-begin-type}detective{-end-type}
      [enigma] {-begin-type}enigma{-end-type}
      [explorer] {-begin-type}explorer{-end-type}
      [hacker] {-begin-type}hacker{-end-type}
      [mage] {-begin-type}mage{-end-type}
      [monster] {-begin-type}monster{-end-type}
      [musician] {-begin-type}musician{-end-type}
      [outsider] {-begin-type}outsider{-end-type}
      [renegade] {-begin-type}renegade{-end-type}
      [spirit-animal] {-begin-type}spirit animal{-end-type}
      [super] {-begin-type}super{-end-type}
      [survivor] {-begin-type}survivor{-end-type}
      [synth] {-begin-type}synth{-end-type}
      [tinkerer] {-begin-type}tinkerer{-end-type}
      [trooper] {-begin-type}trooper{-end-type}
      [visionary] {-begin-type}visionary{-end-type}
      [visitor] {-begin-type}visitor{-end-type}
      [warrior] {-begin-type}warrior{-end-type}
      *[other] Error: Unknown 'type' for type: { $type }
  }

plural-subtype =
  {
    $type ->
      [ancient] {-begin-type}ancients{-end-type}
      [child] {-begin-type}children{-end-type}
      [detective] {-begin-type}detectives{-end-type}
      [enigma] {-begin-type}enigmas{-end-type}
      [explorer] {-begin-type}explorers{-end-type}
      [hacker] {-begin-type}hackers{-end-type}
      [mage] {-begin-type}mages{-end-type}
      [monster] {-begin-type}monsters{-end-type}
      [musician] {-begin-type}musicians{-end-type}
      [outsider] {-begin-type}outsiders{-end-type}
      [renegade] {-begin-type}renegades{-end-type}
      [spirit-animal] {-begin-type}spirit animals{-end-type}
      [super] {-begin-type}supers{-end-type}
      [survivors] {-begin-type}survivors{-end-type}
      [synths] {-begin-type}synths{-end-type}
      [tinkerers] {-begin-type}tinkerers{-end-type}
      [troopers] {-begin-type}troopers{-end-type}
      [visionaries] {-begin-type}visionaries{-end-type}
      [visitors] {-begin-type}visitors{-end-type}
      [warrior] {-begin-type}warriors{-end-type}
      *[other] Error: Unknown 'plural-type' for type: { $type }
  }