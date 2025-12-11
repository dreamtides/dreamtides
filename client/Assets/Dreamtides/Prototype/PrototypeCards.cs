#nullable enable

using System;
using System.Collections.Generic;
using Dreamtides.Masonry;
using Dreamtides.Schema;

namespace Dreamtides.Prototype
{
  [Serializable]
  // Per-card customization for creation/update requests.
  public class CardOverride
  {
    // Zero-based index within the requested group window
    public int Index { get; set; }

    // Optional prefab override
    public CardPrefab? Prefab { get; set; }

    // Optional revealed data overrides
    public string? Name { get; set; }
    public string? Cost { get; set; }
    public string? Rules { get; set; }
    public string? CardType { get; set; }
    public bool? IsFast { get; set; }
    public long? ImageNumber { get; set; }
    public string? SpritePath { get; set; }
    public string? Spark { get; set; }
    public string? Produced { get; set; }
    public string? OutlineColorHex { get; set; }
    public string? ButtonAttachmentLabel { get; set; }
  }

  public class CreateOrUpdateCardsRequest
  {
    public int Count { get; set; }
    public ObjectPosition Position { get; set; } = null!;
    public bool Revealed { get; set; } = true;
    public string? OutlineColorHex { get; set; }
    public string GroupKey { get; set; } = "default";

    // Optional: number of cards (starting from index 0) to force using the Dreamsign prefab
    public int DreamsignPrefabCount { get; set; } = 0;

    // Optional: specific zero-based indices to use Dreamsign prefab (takes precedence over DreamsignPrefabCount)
    public int[]? DreamsignPrefabIndices { get; set; }

    // Optional: arbitrary per-index overrides applied after templating (takes precedence over template fields)
    public List<CardOverride>? Overrides { get; set; }

    // When provided, attach a card on-click action that triggers a debug test scenario
    // with this string. Action path: action.Value.GameActionClass?.DebugAction?.DebugActionClass?.ApplyTestScenarioAction
    public string? OnClickDebugScenario { get; set; }

    public string? ButtonAttachmentLabel { get; set; }

    public string? ButtonAttachmentDebugScenario { get; set; }
  }

  public class PrototypeCards
  {
    // Persistent per-group caches of created cards (never shrink per group).
    readonly Dictionary<string, List<CardView>> _groupCaches =
      new Dictionary<string, List<CardView>>();

    // Fixed base seed for deterministic per-index pseudo-random selection (instance-level, not static).
    readonly int _baseSeed = 0x5EEDBEEF; // Arbitrary constant

    public PrototypeCards() { }

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
        // Unified override path (including dreamsign indices)
        var cardOv = GetEffectiveOverride(request, i);
        CardPrefab? prefabOverride = cardOv?.Prefab;
        var card = BuildCardView(
          request.GroupKey,
          template,
          objectPosition,
          i,
          request.Revealed,
          request.OutlineColorHex,
          prefabOverride
        );
        if (request.Revealed && card.Revealed != null && cardOv != null)
        {
          ApplyRevealedOverrides(card.Revealed, cardOv);
        }
        if (request.Revealed && card.Revealed != null)
        {
          ConfigureCardActionsIfRequested(card.Revealed, card.Id, request, cardOv);
        }
        cache.Add(card);
      }

      // Update (only) the first 'count' cards' positions (and facing/revealed state) to reflect the new base position request.
      // Remaining cards keep prior positions exactly as required by the spec example (20 -> 4 returns 20 with only first 4 moved).
      for (int i = 0; i < Math.Min(request.Count, cache.Count); i++)
      {
        var existing = cache[i];
        var templateForUpdate = GetTemplateForIndex(request.GroupKey, i);
        var cardOv = GetEffectiveOverride(request, i);
        existing.Position = ClonePositionWithSorting(request.Position, i); // Update sorting key relative to new request
        if (existing.CardFacing != (request.Revealed ? CardFacing.FaceUp : CardFacing.FaceDown))
        {
          existing.CardFacing = request.Revealed ? CardFacing.FaceUp : CardFacing.FaceDown;
        }
        // Keep prefab consistent via effective overrides (which include Dreamsign indices)
        existing.Prefab = cardOv?.Prefab ?? templateForUpdate.Prefab;
        existing.Backless = IsBacklessPrefab(existing.Prefab);
        if (request.Revealed)
        {
          if (existing.Revealed == null)
          {
            // If previously hidden but now revealed, build a revealed view deterministically from its template index.
            existing.Revealed = BuildRevealed(templateForUpdate, request.OutlineColorHex);
          }
          else if (request.OutlineColorHex != null)
          {
            // Update outline color if provided.
            existing.Revealed.OutlineColor = Mason.MakeColor(request.OutlineColorHex);
          }

          // Apply any per-card revealed overrides (take precedence over request outline)
          if (cardOv != null && existing.Revealed != null)
          {
            ApplyRevealedOverrides(existing.Revealed, cardOv);
          }

          if (existing.Revealed != null)
          {
            ConfigureCardActionsIfRequested(existing.Revealed, existing.Id, request, cardOv);
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

    static bool IsBacklessPrefab(CardPrefab prefab) =>
      prefab == CardPrefab.Dreamsign
      || prefab == CardPrefab.IconCard
      || prefab == CardPrefab.Journey
      || prefab == CardPrefab.OfferCost;

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
        var rng = new System.Random(seed);
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
      string? outlineColorHex,
      CardPrefab? overridePrefab = null
    ) =>
      new()
      {
        Backless = IsBacklessPrefab(overridePrefab ?? t.Prefab),
        CardFacing = revealed ? CardFacing.FaceUp : CardFacing.FaceDown,
        Id = BuildId(groupKey, sortIndex),
        Position = objectPosition,
        Prefab = overridePrefab ?? t.Prefab,
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

    OnClickUnion BuildDebugOnClick(string cardId, string scenario) =>
      new OnClickUnion
      {
        OnClickClass = new OnClickClass
        {
          DebugAction = new DebugAction
          {
            DebugActionClass = new DebugActionClass
            {
              ApplyTestScenarioAction = $@"{scenario}/{cardId}",
            },
          },
        },
      };

    ButtonView BuildButtonAttachment(string cardId, string label, string? scenario) =>
      new ButtonView
      {
        Label = label,
        Action = scenario != null ? BuildDebugOnClick(cardId, scenario) : null,
      };

    void ConfigureCardActionsIfRequested(
      RevealedCardView revealed,
      string cardId,
      CreateOrUpdateCardsRequest request,
      CardOverride? overrideForIndex
    )
    {
      EnsureActions(revealed);
      if (request.OnClickDebugScenario != null)
      {
        revealed.Actions.OnClick = BuildDebugOnClick(cardId, request.OnClickDebugScenario);
      }
      var label = overrideForIndex?.ButtonAttachmentLabel ?? request.ButtonAttachmentLabel;
      if (label != null || request.ButtonAttachmentDebugScenario != null)
      {
        revealed.Actions.ButtonAttachment = BuildButtonAttachment(
          cardId,
          label ?? string.Empty,
          request.ButtonAttachmentDebugScenario
        );
      }
    }

    ObjectPosition ClonePositionWithSorting(ObjectPosition basePosition, int sortingKey) =>
      new()
      {
        Position = ClonePosition(basePosition.Position, sortingKey),
        SortingKey = sortingKey,
      };

    Position ClonePosition(Position basePosition, int sortingKey)
    {
      if (basePosition.Enum != null)
      {
        return new Position { Enum = basePosition.Enum };
      }
      if (basePosition.PositionClass != null)
      {
        return new Position
        {
          PositionClass = ClonePositionClass(basePosition.PositionClass, sortingKey),
        };
      }
      return new Position();
    }

    PositionClass ClonePositionClass(PositionClass baseClass, int sortingKey)
    {
      var clone = new PositionClass
      {
        OnStack = baseClass.OnStack,
        InHand = baseClass.InHand,
        InDeck = baseClass.InDeck,
        InVoid = baseClass.InVoid,
        InBanished = baseClass.InBanished,
        OnBattlefield = baseClass.OnBattlefield,
        InPlayerStatus = baseClass.InPlayerStatus,
        CardOrderSelector = baseClass.CardOrderSelector,
        InDreamwell = baseClass.InDreamwell,
        HiddenWithinCard = baseClass.HiddenWithinCard,
        AboveVoid = baseClass.AboveVoid,
        SiteDeck = baseClass.SiteDeck,
        SiteNpc = baseClass.SiteNpc,
        TemptingOfferDisplay = CloneTemptingOfferPosition(
          baseClass.TemptingOfferDisplay,
          sortingKey
        ),
        StartBattleDisplay = baseClass.StartBattleDisplay,
      };
      return clone;
    }

    TemptingOfferPosition? CloneTemptingOfferPosition(
      TemptingOfferPosition? basePosition,
      int sortingKey
    )
    {
      if (basePosition == null)
      {
        return null;
      }
      var offerType = sortingKey % 2 == 0 ? TemptingOfferType.Journey : TemptingOfferType.Cost;
      var offerNumber = sortingKey / 2;
      return new TemptingOfferPosition { Number = offerNumber, OfferType = offerType };
    }

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

    bool IsDreamsignIndex(CreateOrUpdateCardsRequest request, int index)
    {
      var indices = request.DreamsignPrefabIndices;
      if (indices != null && indices.Length > 0)
      {
        for (int i = 0; i < indices.Length; i++)
        {
          if (indices[i] == index)
          {
            return true;
          }
        }
        return false;
      }
      return index < request.DreamsignPrefabCount;
    }

    CardOverride? GetOverrideForIndex(CreateOrUpdateCardsRequest request, int index)
    {
      var overrides = request.Overrides;
      if (overrides == null || overrides.Count == 0)
        return null;
      for (int i = 0; i < overrides.Count; i++)
      {
        if (overrides[i].Index == index)
          return overrides[i];
      }
      return null;
    }

    // Merge Dreamsign index handling into the overrides path without mutating the caller's overrides.
    CardOverride? GetEffectiveOverride(CreateOrUpdateCardsRequest request, int index)
    {
      var ov = GetOverrideForIndex(request, index);
      var isDream = IsDreamsignIndex(request, index);
      if (!isDream)
      {
        return ov;
      }

      if (ov == null)
      {
        return new CardOverride { Index = index, Prefab = CardPrefab.Dreamsign };
      }

      if (ov.Prefab == null)
      {
        // Return a shallow copy with Prefab set to Dreamsign so we don't mutate the original
        return new CardOverride
        {
          Index = ov.Index,
          Prefab = CardPrefab.Dreamsign,
          Name = ov.Name,
          Cost = ov.Cost,
          Rules = ov.Rules,
          CardType = ov.CardType,
          IsFast = ov.IsFast,
          ImageNumber = ov.ImageNumber,
          SpritePath = ov.SpritePath,
          Spark = ov.Spark,
          Produced = ov.Produced,
          OutlineColorHex = ov.OutlineColorHex,
        };
      }

      return ov;
    }

    void ApplyRevealedOverrides(RevealedCardView revealed, CardOverride ov)
    {
      if (ov.Name != null)
        revealed.Name = ov.Name;
      if (ov.Cost != null)
        revealed.Cost = ov.Cost;
      if (ov.Rules != null)
        revealed.RulesText = ov.Rules;
      if (ov.CardType != null)
        revealed.CardType = ov.CardType;
      if (ov.IsFast.HasValue)
        revealed.IsFast = ov.IsFast.Value;
      if (ov.Spark != null)
        revealed.Spark = ov.Spark;
      if (ov.Produced != null)
        revealed.Produced = ov.Produced;
      if (!string.IsNullOrEmpty(ov.OutlineColorHex))
      {
        revealed.OutlineColor = Mason.MakeColor(ov.OutlineColorHex);
      }
      if (!string.IsNullOrEmpty(ov.SpritePath))
      {
        revealed.Image = new DisplayImage { Sprite = new SpriteAddress { Sprite = ov.SpritePath } };
      }
      else if (ov.ImageNumber.HasValue)
      {
        revealed.Image = new DisplayImage
        {
          Sprite = new SpriteAddress { Sprite = BuildSpritePath(ov.ImageNumber.Value) },
        };
      }
    }

    // Clear a group's cache so that subsequent requests will recreate cards from templates.
    public void ResetGroup(string groupKey)
    {
      if (_groupCaches.ContainsKey(groupKey))
      {
        _groupCaches.Remove(groupKey);
      }
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

    public List<CardView> GetAllCards()
    {
      var result = new List<CardView>();
      foreach (var kvp in _groupCaches)
      {
        result.AddRange(kvp.Value);
      }
      return result;
    }

    public List<CardView> GetGroupCards(string groupKey)
    {
      if (_groupCaches.TryGetValue(groupKey, out var cache))
      {
        return new List<CardView>(cache);
      }
      return new List<CardView>();
    }

    #endregion
  }
}
