#nullable enable

using System.Collections.Generic;
using Dreamtides.Schema;
using System;

namespace Dreamtides.Prototype
{
  public static class PrototypeCards
  {
    public static List<CardView> CreateCards(int count, ObjectPosition position, bool revealed = true)
    {
      if (count <= 0) return new List<CardView>();
      if (position == null) throw new ArgumentNullException(nameof(position));

      var list = new List<CardView>(capacity: count);
      var rng = _rng;

      for (var i = 0; i < count; i++)
      {
        var template = _cardTemplates[rng.Next(_cardTemplates.Length)];
        var objectPosition = ClonePositionWithSorting(position, i);
        list.Add(BuildCardView(template, objectPosition, i, revealed));
      }

      return list;
    }

    #region Helpers

    // Single shared RNG so repeated calls feel varied but deterministic within a run if desired.
    static readonly Random _rng = new Random();

    // Minimal template info needed to fabricate a revealed card view.
    class CardTemplate
    {
      public string Name { get; set; }
      public string Cost { get; set; }
      public string Rules { get; set; }
      public string CardType { get; set; }
      public CardPrefab Prefab { get; set; }
      public bool IsFast { get; set; }
      public long ImageNumber { get; set; }
      public string? Spark { get; set; } // optional, may be null
      public string? Produced { get; set; } // optional, may be null

      public CardTemplate(string name, string cost, string rules, string cardType, CardPrefab prefab, bool isFast, long imageNumber, string? spark = null, string? produced = null)
      {
        Name = name;
        Cost = cost;
        Rules = rules;
        CardType = cardType;
        Prefab = prefab;
        IsFast = isFast;
        ImageNumber = imageNumber;
        Spark = spark;
        Produced = produced;
      }
    }

    static readonly CardTemplate[] _cardTemplates = new CardTemplate[]
    {
      new CardTemplate(
        name: "Immolate",
        cost: "2",
        rules: "{Dissolve} an enemy character.",
        cardType: "Event",
        prefab: CardPrefab.Event,
        isFast: true,
        imageNumber: 1907487244
      ),
      new CardTemplate(
        name: "Abolish",
        cost: "2",
        rules: "{Prevent} a played enemy card.",
        cardType: "Event",
        prefab: CardPrefab.Event,
        isFast: true,
        imageNumber: 1282908322
      ),
      new CardTemplate(
        name: "Ripple of Defiance",
        cost: "1",
        rules: "{Prevent} a played enemy event unless the enemy pays {-energy-cost(e:2)}.",
        cardType: "Event",
        prefab: CardPrefab.Event,
        isFast: true,
        imageNumber: 2123360837
      ),
      new CardTemplate(
        name: "Dreamscatter",
        cost: "2",
        rules: "Pay one or more {e}: Draw {-drawn-cards(n:1)} for each {e} spent.",
        cardType: "Event",
        prefab: CardPrefab.Event,
        isFast: true,
        imageNumber: 489056605
      ),
      new CardTemplate(
        name: "Sundown Surfer",
        cost: "2",
        rules: "Whenever you play a card during the enemy's turn, this character gains {-gained-spark(n:1)}.",
        cardType: "Character",
        prefab: CardPrefab.Character,
        isFast: true,
        imageNumber: 403770319,
        spark: "1"
      ),
      new CardTemplate(
        name: "Minstrel of Falling Light",
        cost: "2",
        rules: "{fma} {-energy-cost(e:3)}: Draw {-drawn-cards(n:1)}.",
        cardType: "Character",
        prefab: CardPrefab.Character,
        isFast: false,
        imageNumber: 1794244540,
        spark: "2"
      ),
      new CardTemplate(
        name: "Archive of the Forgotten",
        cost: "4",
        rules: "Return one or two events from your void to your hand.",
        cardType: "Event",
        prefab: CardPrefab.Event,
        isFast: false,
        imageNumber: 644603677
      ),
      new CardTemplate(
        name: "Together Against the Tide",
        cost: "1",
        rules: "Give an allied character {anchored} until end of turn.",
        cardType: "Event",
        prefab: CardPrefab.Event,
        isFast: true,
        imageNumber: 1621160806
      ),
      new CardTemplate(
        name: "Guiding Light",
        cost: "1",
        rules: "{-Foresee(n:1)}. Draw {-drawn-cards(n:1)}.\n\n{-Reclaim-Cost(e:3)}",
        cardType: "Event",
        prefab: CardPrefab.Event,
        isFast: true,
        imageNumber: 1328168243
      ),
      new CardTemplate(
        name: "Cragfall",
        cost: "2",
        rules: "{Prevent} a played enemy character.",
        cardType: "Event",
        prefab: CardPrefab.Event,
        isFast: true,
        imageNumber: 1239919309
      )
    };

    static CardView BuildCardView(CardTemplate t, ObjectPosition objectPosition, int sortIndex, bool revealed) => new()
    {
      Backless = revealed, // Only backless if revealed so we don't animate a flip
      CardFacing = revealed ? CardFacing.FaceUp : CardFacing.FaceDown,
      Id = (sortIndex + 1).ToString(),
      Position = objectPosition,
      Prefab = t.Prefab,
      Revealed = revealed ? BuildRevealed(t) : null,
      RevealedToOpponents = true,
      // Optional fields left null to intentionally avoid outlines / effects / actions
    };

    static RevealedCardView BuildRevealed(CardTemplate t) => new()
    {
      Actions = new CardActions(), // All null => no actions, clicks, sounds
      CardType = t.CardType,
      Cost = t.Cost,
      Effects = new CardEffects(), // LoopingEffect left null => no visual effects
      Image = new DisplayImage
      {
        Sprite = new SpriteAddress { Sprite = BuildSpritePath(t.ImageNumber) }
      },
      InfoZoomData = null,
      IsFast = t.IsFast,
      Name = t.Name,
      OutlineColor = null, // Explicit for clarity (no outline override)
      Produced = t.Produced,
      RulesText = t.Rules,
      Spark = t.Spark
    };

    static ObjectPosition ClonePositionWithSorting(ObjectPosition basePosition, int sortingKey) => new()
    {
      Position = basePosition.Position,
      SortingKey = sortingKey
    };

    static string BuildSpritePath(long imageNumber) =>
      $"Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_{imageNumber}.png";

    #endregion
  }
}