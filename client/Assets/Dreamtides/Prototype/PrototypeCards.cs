#nullable enable

using System;
using System.Collections.Generic;
using Dreamtides.Masonry;
using Dreamtides.Schema;

namespace Dreamtides.Prototype
{
  public class CreateOrUpdateCardsRequest
  {
    public int Count { get; set; }
    public ObjectPosition Position { get; set; } = null!;
    public bool Revealed { get; set; } = true;
    public string? OutlineColorHex { get; set; }
    public string GroupKey { get; set; } = "default";

    // When provided, attach a card on-click action that triggers a debug test scenario
    // with this string. Action path: action.Value.GameActionClass?.DebugAction?.DebugActionClass?.ApplyTestScenarioAction
    public string? OnClickDebugScenario { get; set; }
  }

  public class PrototypeCards
  {
    /// <summary>
    /// Per-group deterministic card creation and updates.
    /// - Maintains an independent cache for each <paramref name="groupKey"/> (e.g., "quest", "draft").
    /// - For the specified group, grows (never shrinks) up to at least <paramref name="count"/>, and updates the first N cards' positions
    ///   (and optionally facing/revealed state).
    /// - Returns the union of ALL groups' cached cards with the requested group's cards first, so callers can safely pass the result
    ///   to a full-state updater without deleting cards from other groups, while still being able to take the first N cards for
    ///   animations in the active group.
    /// Card identities are group-namespaced (e.g., "draft-1"), ensuring no ID collisions across groups.
    /// </summary>
    public List<CardView> CreateOrUpdateCards(CreateOrUpdateCardsRequest request)
    {
      if (request == null)
        throw new ArgumentNullException(nameof(request));
      if (request.Position == null)
        throw new ArgumentNullException(nameof(request.Position));
      if (request.Count <= 0)
      {
        // Preserve previous behavior of returning empty when asked for <= 0.
        return new List<CardView>();
      }

      // Ensure the group's cache exists.
      if (!_groupCaches.TryGetValue(request.GroupKey, out var cache))
      {
        cache = new List<CardView>();
        _groupCaches[request.GroupKey] = cache;
      }

      // Grow this group's cache if needed (never shrink).
      var targetCount = Math.Max(request.Count, cache.Count);

      // Generate any missing cards deterministically by index (seed derived from global seed + card index) so order/content
      // is independent of the sequence of external calls.
      for (int i = cache.Count; i < targetCount; i++)
      {
        var template = GetTemplateForIndex(request.GroupKey, i);
        var objectPosition = ClonePositionWithSorting(request.Position, i);
        var card = BuildCardView(
          request.GroupKey,
          template,
          objectPosition,
          i,
          request.Revealed,
          request.OutlineColorHex
        );
        if (request.Revealed && request.OnClickDebugScenario != null && card.Revealed != null)
        {
          EnsureActions(card.Revealed);
          var cardId = card.Id; // use actual id to avoid mismatches if ordering changes
          card.Revealed.Actions.OnClick = BuildDebugOnClick(cardId, request.OnClickDebugScenario);
        }
        cache.Add(card);
      }

      // Update (only) the first 'count' cards' positions (and facing/revealed state) to reflect the new base position request.
      // Remaining cards keep prior positions exactly as required by the spec example (20 -> 4 returns 20 with only first 4 moved).
      for (int i = 0; i < Math.Min(request.Count, cache.Count); i++)
      {
        var existing = cache[i];
        existing.Position = ClonePositionWithSorting(request.Position, i); // Update sorting key relative to new request
        if (existing.CardFacing != (request.Revealed ? CardFacing.FaceUp : CardFacing.FaceDown))
        {
          existing.CardFacing = request.Revealed ? CardFacing.FaceUp : CardFacing.FaceDown;
        }
        if (request.Revealed)
        {
          if (existing.Revealed == null)
          {
            // If previously hidden but now revealed, build a revealed view deterministically from its template index.
            existing.Revealed = BuildRevealed(
              GetTemplateForIndex(request.GroupKey, i),
              request.OutlineColorHex
            );
          }
          else if (request.OutlineColorHex != null)
          {
            // Update outline color if provided.
            existing.Revealed.OutlineColor = Mason.MakeColor(request.OutlineColorHex);
          }

          // Wire up on-click debug scenario for updated cards when requested
          if (request.OnClickDebugScenario != null)
          {
            EnsureActions(existing.Revealed);
            var cardId = existing.Id; // do not recompute by index; cache may be rotated
            existing.Revealed.Actions.OnClick = BuildDebugOnClick(
              cardId,
              request.OnClickDebugScenario
            );
          }
        }
        else // !revealed
        {
          existing.Revealed = null; // Conceal if request indicates hidden.
        }
      }

      // Build a union list of all groups with requested group's cards first so callers keep other groups alive on update.
      var result = new List<CardView>(cache.Count + TotalOtherCardsCount(request.GroupKey));
      result.AddRange(cache);
      foreach (var kvp in _groupCaches)
      {
        if (kvp.Key == request.GroupKey)
          continue;
        result.AddRange(kvp.Value);
      }

      // Return a defensive copy so callers cannot mutate the caches.
      return result;
    }

    #region Helpers

    // Persistent per-group caches of created cards (never shrink per group).
    readonly Dictionary<string, List<CardView>> _groupCaches =
      new Dictionary<string, List<CardView>>();

    // Fixed base seed for deterministic per-index pseudo-random selection (instance-level, not static).
    readonly int _baseSeed = 0x5EEDBEEF; // Arbitrary constant

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

      public CardTemplate(
        string name,
        string cost,
        string rules,
        string cardType,
        CardPrefab prefab,
        bool isFast,
        long imageNumber,
        string? spark = null,
        string? produced = null
      )
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

    readonly CardTemplate[] _cardTemplates = new CardTemplate[]
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
      ),
    };

    // Deterministically pick a template for a given group+index independent of call order.
    CardTemplate GetTemplateForIndex(string groupKey, int index)
    {
      // Derive a per-card seed; use unchecked to allow overflow wrapping.
      unchecked
      {
        int seed = _baseSeed ^ StableHash(groupKey) + index * 31; // group-scoped deterministic seed
        var rng = new Random(seed);
        int templateIndex = rng.Next(_cardTemplates.Length);
        return _cardTemplates[templateIndex];
      }
    }

    CardView BuildCardView(
      string groupKey,
      CardTemplate t,
      ObjectPosition objectPosition,
      int sortIndex,
      bool revealed,
      string? outlineColorHex
    ) =>
      new()
      {
        Backless = false,
        CardFacing = revealed ? CardFacing.FaceUp : CardFacing.FaceDown,
        Id = BuildId(groupKey, sortIndex),
        Position = objectPosition,
        Prefab = t.Prefab,
        Revealed = revealed ? BuildRevealed(t, outlineColorHex) : null,
        RevealedToOpponents = false,
      };

    RevealedCardView BuildRevealed(CardTemplate t, string? outlineColorHex) =>
      new()
      {
        Actions = new CardActions(), // All null => no actions, clicks, sounds
        CardType = t.CardType,
        Cost = t.Cost,
        Effects = new CardEffects(), // LoopingEffect left null => no visual effects
        Image = new DisplayImage
        {
          Sprite = new SpriteAddress { Sprite = BuildSpritePath(t.ImageNumber) },
        },
        InfoZoomData = null,
        IsFast = t.IsFast,
        Name = t.Name,
        OutlineColor = outlineColorHex != null ? Mason.MakeColor(outlineColorHex) : null,
        Produced = t.Produced,
        RulesText = t.Rules,
        Spark = t.Spark,
      };

    void EnsureActions(RevealedCardView revealed)
    {
      if (revealed.Actions == null)
      {
        revealed.Actions = new CardActions();
      }
    }

    ActionUnion BuildDebugOnClick(string cardId, string scenario) =>
      new ActionClass
      {
        DebugAction = new DebugAction
        {
          DebugActionClass = new DebugActionClass
          {
            ApplyTestScenarioAction = $@"{scenario}/{cardId}",
          },
        },
      };

    ObjectPosition ClonePositionWithSorting(ObjectPosition basePosition, int sortingKey) =>
      new() { Position = basePosition.Position, SortingKey = sortingKey };

    string BuildSpritePath(long imageNumber) =>
      $"Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_{imageNumber}.png";

    string BuildId(string groupKey, int index) => $"{groupKey}-{index + 1}";

    int StableHash(string s)
    {
      unchecked
      {
        int h = 23;
        for (int i = 0; i < s.Length; i++)
        {
          h = h * 31 + s[i];
        }
        return h;
      }
    }

    int TotalOtherCardsCount(string currentGroup)
    {
      int total = 0;
      foreach (var kvp in _groupCaches)
      {
        if (kvp.Key == currentGroup)
          continue;
        total += kvp.Value.Count;
      }
      return total;
    }

    public void AdvanceGroupWindow(string groupKey, int by)
    {
      if (!_groupCaches.TryGetValue(groupKey, out var cache))
      {
        return;
      }
      if (cache.Count == 0)
      {
        return;
      }
      by %= cache.Count;
      if (by < 0)
        by += cache.Count;
      if (by == 0)
      {
        return;
      }

      var rotated = new List<CardView>(cache.Count);
      for (int i = 0; i < cache.Count; i++)
      {
        int src = (i + by) % cache.Count;
        rotated.Add(cache[src]);
      }
      _groupCaches[groupKey] = rotated;
    }

    public void UpdateGroupCards(string groupKey, IEnumerable<CardView> updates)
    {
      if (!_groupCaches.TryGetValue(groupKey, out var cache))
      {
        return;
      }

      foreach (var updated in updates)
      {
        for (int i = 0; i < cache.Count; i++)
        {
          if (cache[i].Id == updated.Id)
          {
            var current = cache[i];
            current.Position = updated.Position;
            current.CardFacing = updated.CardFacing;
            current.Revealed = updated.Revealed;
            current.RevealedToOpponents = updated.RevealedToOpponents;
            cache[i] = current;
            break;
          }
        }
      }
    }

    #endregion
  }
}
