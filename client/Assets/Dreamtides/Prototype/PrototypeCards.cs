#nullable enable

using System.Collections.Generic;
using Dreamtides.Schema;
using System;
using Dreamtides.Masonry;

namespace Dreamtides.Prototype
{
  public static class PrototypeCards
  {
    /// <summary>
    /// Create new cards up to at least <paramref name="count"/> total, or update the first <paramref name="count"/> existing cards' positions.
    /// Subsequent calls with a smaller count DO NOT shrink the underlying cached collection; they only adjust the positions
    /// (and optionally facing/revealed state) of the first N cards. This allows a call sequence like (20) then (4) to still
    /// return 20 cards, with only the first 4 repositioned as requested.
    /// Card generation is fully deterministic: requesting (count = 20) will always yield the same 20 card identities & templates
    /// regardless of any prior calls with different counts or parameters.
    /// </summary>
    public static List<CardView> CreateOrUpdateCards(int count, ObjectPosition position, bool revealed = true, string? outlineColorHex = null)
    {
      if (position == null) throw new ArgumentNullException(nameof(position));
      if (count <= 0)
      {
        // Preserve previous behavior of returning empty when asked for <= 0.
        return new List<CardView>();
      }

      // Grow cache if needed (never shrink) so that subsequent smaller requests still return the full prior set.
      var targetCount = Math.Max(count, _cachedCards.Count);

      // Generate any missing cards deterministically by index (seed derived from global seed + card index) so order/content
      // is independent of the sequence of external calls.
      for (int i = _cachedCards.Count; i < targetCount; i++)
      {
        var template = GetTemplateForIndex(i);
        var objectPosition = ClonePositionWithSorting(position, i);
        _cachedCards.Add(BuildCardView(template, objectPosition, i, revealed, outlineColorHex));
      }

      // Update (only) the first 'count' cards' positions (and facing/revealed state) to reflect the new base position request.
      // Remaining cards keep prior positions exactly as required by the spec example (20 -> 4 returns 20 with only first 4 moved).
      for (int i = 0; i < Math.Min(count, _cachedCards.Count); i++)
      {
        var existing = _cachedCards[i];
        existing.Position = ClonePositionWithSorting(position, i); // Update sorting key relative to new request
        if (existing.CardFacing != (revealed ? CardFacing.FaceUp : CardFacing.FaceDown))
        {
          existing.CardFacing = revealed ? CardFacing.FaceUp : CardFacing.FaceDown;
        }
        if (revealed)
        {
          if (existing.Revealed == null)
          {
            // If previously hidden but now revealed, build a revealed view deterministically from its template index.
            existing.Revealed = BuildRevealed(GetTemplateForIndex(i), outlineColorHex);
          }
          else if (outlineColorHex != null)
          {
            // Update outline color if provided.
            existing.Revealed.OutlineColor = Mason.MakeColor(outlineColorHex);
          }
        }
        else // !revealed
        {
          existing.Revealed = null; // Conceal if request indicates hidden.
        }
      }

      // Return a defensive copy so callers cannot mutate the cached list.
      return new List<CardView>(_cachedCards);
    }

    #region Helpers

    // Persistent cache of created cards (never shrinks).
    static readonly List<CardView> _cachedCards = new List<CardView>();

    // Fixed base seed for deterministic per-index pseudo-random selection.
    const int _baseSeed = 0x5EEDBEEF; // Arbitrary constant

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

    // Deterministically pick a template for a given card index independent of call order.
    static CardTemplate GetTemplateForIndex(int index)
    {
      // Derive a per-card seed; use unchecked to allow overflow wrapping.
      unchecked
      {
        int seed = _baseSeed + index * 31; // 31 is a small prime multiplier
        var rng = new Random(seed);
        int templateIndex = rng.Next(_cardTemplates.Length);
        return _cardTemplates[templateIndex];
      }
    }

    static CardView BuildCardView(CardTemplate t, ObjectPosition objectPosition, int sortIndex, bool revealed, string? outlineColorHex) => new()
    {
      Backless = false,
      CardFacing = revealed ? CardFacing.FaceUp : CardFacing.FaceDown,
      Id = (sortIndex + 1).ToString(),
      Position = objectPosition,
      Prefab = t.Prefab,
      Revealed = revealed ? BuildRevealed(t, outlineColorHex) : null,
      RevealedToOpponents = false,
    };

    static RevealedCardView BuildRevealed(CardTemplate t, string? outlineColorHex) => new()
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
      OutlineColor = outlineColorHex != null ? Mason.MakeColor(outlineColorHex) : null,
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