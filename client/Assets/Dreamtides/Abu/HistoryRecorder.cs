#nullable enable

using System.Collections.Generic;
using Abu;
using Dreamtides.Schema;
using Dreamtides.Services;

namespace Dreamtides.Abu
{
  /// <summary>
  /// Records game events by observing commands flowing through ActionService
  /// and produces concise history entries for Abu action responses.
  /// </summary>
  public class HistoryRecorder : IHistoryProvider
  {
    readonly CardService _cardService;
    readonly List<string> _entries = new();
    readonly Dictionary<string, TrackedCard> _cardPositions = new();

    struct TrackedCard
    {
      public string Zone;
      public DisplayPlayer? Player;
      public string? Name;
    }

    public HistoryRecorder(CardService cardService)
    {
      _cardService = cardService;
    }

    /// <summary>
    /// Called for each command processed by ActionService.
    /// </summary>
    public void OnCommand(Command command)
    {
      if (command.DisplayGameMessage != null)
      {
        RecordGameMessage(command.DisplayGameMessage.Value);
      }

      if (command.DisplayDreamwellActivation != null)
      {
        RecordDreamwellActivation(command.DisplayDreamwellActivation);
      }

      if (command.ShuffleVoidIntoDeck != null)
      {
        RecordShuffleVoidIntoDeck(command.ShuffleVoidIntoDeck);
      }

      if (command.UpdateBattle != null)
      {
        RecordCardTransitions(command.UpdateBattle.Battle);
      }
    }

    public void ClearHistory()
    {
      _entries.Clear();
    }

    public List<string>? TakeHistory()
    {
      if (_entries.Count == 0)
      {
        return null;
      }

      var result = new List<string>(_entries);
      _entries.Clear();
      return result;
    }

    void RecordGameMessage(GameMessageType messageType)
    {
      var text = messageType switch
      {
        GameMessageType.YourTurn => "Your turn begins",
        GameMessageType.EnemyTurn => "Opponent's turn begins",
        GameMessageType.Victory => "Victory!",
        GameMessageType.Defeat => "Defeat",
        _ => null,
      };

      if (text != null)
      {
        _entries.Add(text);
      }
    }

    void RecordDreamwellActivation(DisplayDreamwellActivationCommand activation)
    {
      var cardName = "Unknown";
      var rulesText = "";

      var card = _cardService.GetCardIfExists(activation.CardId);
      if (card != null)
      {
        var revealed = card.CardView.Revealed;
        if (revealed != null)
        {
          cardName = DreamtidesSceneWalker.StripRichText(revealed.Name);
          rulesText = DreamtidesSceneWalker.StripRichText(revealed.RulesText);
        }
      }

      var prefix = activation.Player == DisplayPlayer.User ? "Dreamwell" : "Enemy dreamwell";
      var displayRulesText = string.IsNullOrEmpty(rulesText) ? "(no rules text)" : rulesText;
      var entry = $"{prefix}: {cardName} -- {displayRulesText}";
      _entries.Add(entry);
    }

    void RecordShuffleVoidIntoDeck(ShuffleVoidIntoDeckCommand shuffle)
    {
      _entries.Add(
        shuffle.Player == DisplayPlayer.User
          ? "Void shuffled into deck"
          : "Enemy void shuffled into deck"
      );
    }

    void RecordCardTransitions(BattleView battle)
    {
      var newPositions = new Dictionary<string, TrackedCard>();

      foreach (var cardView in battle.Cards)
      {
        var zone = ClassifyZone(cardView.Position.Position);
        var player = GetPlayer(cardView.Position.Position);
        var name =
          cardView.Revealed != null
            ? DreamtidesSceneWalker.StripRichText(cardView.Revealed.Name)
            : null;

        newPositions[cardView.Id] = new TrackedCard
        {
          Zone = zone,
          Player = player,
          Name = name,
        };

        if (_cardPositions.TryGetValue(cardView.Id, out var old) && old.Zone != zone)
        {
          var entry = BuildTransitionEntry(
            old,
            new TrackedCard
            {
              Zone = zone,
              Player = player,
              Name = name,
            }
          );
          if (entry != null)
          {
            _entries.Add(entry);
          }
        }
      }

      // Check for cards present in old state but absent in new state
      foreach (var kvp in _cardPositions)
      {
        if (!newPositions.ContainsKey(kvp.Key))
        {
          var cardName = kvp.Value.Name ?? "a card";
          if (!IsInternalZone(kvp.Value.Zone))
          {
            _entries.Add($"{cardName} removed");
          }
        }
      }

      _cardPositions.Clear();
      foreach (var kvp in newPositions)
      {
        _cardPositions[kvp.Key] = kvp.Value;
      }
    }

    string? BuildTransitionEntry(TrackedCard old, TrackedCard current)
    {
      var from = old.Zone;
      var to = current.Zone;

      // Skip transitions involving internal positions
      if (IsInternalZone(from) || IsInternalZone(to))
      {
        return null;
      }

      // Skip DreamwellActivation back to Dreamwell (routine animation return)
      if (from == "dreamwell-activation" && to == "dreamwell")
      {
        return null;
      }

      // Skip Drawn to Hand (routine draw animation finish)
      if (from == "drawn" && to == "hand")
      {
        return null;
      }

      // Skip HandStorage to Hand (routine hand storage return)
      if (from == "hand-storage" && to == "hand")
      {
        return null;
      }

      var name = current.Name ?? old.Name;
      var isRevealed = name != null;

      if (!isRevealed)
      {
        // Face-down card tracking
        return BuildFaceDownEntry(from, to, current.Player ?? old.Player);
      }

      // Revealed card transitions
      if (from == "hand" && to == "stack")
      {
        return $"{name} moved from hand to stack";
      }

      if (from == "stack" && to == "battlefield")
      {
        return $"{name} moved from stack to battlefield";
      }

      if (from == "battlefield" && to == "void")
      {
        return $"{name} moved from battlefield to void";
      }

      if (to == "void")
      {
        return $"{name} moved to void";
      }

      if (to == "banished")
      {
        return $"{name} moved to banished";
      }

      if (from == "void" && to == "hand")
      {
        return $"{name} moved from void to hand";
      }

      if (from == "void" && to == "stack")
      {
        return $"{name} moved from void to stack";
      }

      if (from == "deck" && to == "hand")
      {
        return $"{name} moved from deck to hand";
      }

      return null;
    }

    static string? BuildFaceDownEntry(string from, string to, DisplayPlayer? player)
    {
      if (from == "deck" && to == "hand")
      {
        return player == DisplayPlayer.Enemy ? "Enemy drew a card" : "Drew a card";
      }

      if (to == "void")
      {
        return player == DisplayPlayer.Enemy
          ? "Enemy: a card moved to void"
          : "A card moved to void";
      }

      if (to == "banished")
      {
        return player == DisplayPlayer.Enemy
          ? "Enemy: a card moved to banished"
          : "A card moved to banished";
      }

      return null;
    }

    static string ClassifyZone(Position position)
    {
      if (position.Enum.HasValue)
      {
        return position.Enum.Value switch
        {
          PositionEnum.Drawn => "drawn",
          PositionEnum.DreamwellActivation => "dreamwell-activation",
          PositionEnum.HandStorage => "hand-storage",
          PositionEnum.Offscreen => "offscreen",
          PositionEnum.Default => "default",
          PositionEnum.OnScreenStorage => "on-screen-storage",
          PositionEnum.Browser => "browser",
          PositionEnum.GameModifier => "game-modifier",
          _ => "internal",
        };
      }

      if (position.PositionClass != null)
      {
        var pc = position.PositionClass;
        if (pc.OnStack != null)
          return "stack";
        if (pc.InHand != null)
          return "hand";
        if (pc.InDeck != null)
          return "deck";
        if (pc.InVoid != null)
          return "void";
        if (pc.InBanished != null)
          return "banished";
        if (pc.OnBattlefield != null)
          return "battlefield";
        if (pc.InDreamwell != null)
          return "dreamwell";
        if (pc.InPlayerStatus != null)
          return "player-status";
        if (pc.CardOrderSelector != null)
          return "card-order-selector";
        if (pc.HiddenWithinCard != null)
          return "hidden-within-card";
        if (pc.AboveVoid != null)
          return "above-void";
      }

      return "unknown";
    }

    static DisplayPlayer? GetPlayer(Position position)
    {
      if (position.PositionClass == null)
      {
        return null;
      }

      var pc = position.PositionClass;
      if (pc.InHand != null)
        return pc.InHand;
      if (pc.InDeck != null)
        return pc.InDeck;
      if (pc.InVoid != null)
        return pc.InVoid;
      if (pc.InBanished != null)
        return pc.InBanished;
      if (pc.OnBattlefield != null)
        return pc.OnBattlefield;
      if (pc.InDreamwell != null)
        return pc.InDreamwell;
      if (pc.InPlayerStatus != null)
        return pc.InPlayerStatus;
      if (pc.AboveVoid != null)
        return pc.AboveVoid;
      return null;
    }

    static bool IsInternalZone(string zone)
    {
      return zone
        is "offscreen"
          or "default"
          or "on-screen-storage"
          or "browser"
          or "internal"
          or "unknown"
          or "player-status"
          or "card-order-selector"
          or "hidden-within-card"
          or "game-modifier"
          or "above-void";
    }
  }
}
