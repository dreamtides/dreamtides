#nullable enable

using System.Collections.Generic;
using Abu;
using Dreamtides.Schema;
using Dreamtides.Services;

namespace Dreamtides.Abu
{
  /// <summary>
  /// Records visual effect commands (particles, projectiles, dissolves, trails)
  /// by observing commands flowing through ActionService and produces
  /// human-readable log entries for Abu action responses.
  /// </summary>
  public class EffectLogRecorder : IEffectLogProvider
  {
    readonly CardService _cardService;
    readonly List<string> _entries = new();

    public EffectLogRecorder(CardService cardService)
    {
      _cardService = cardService;
    }

    /// <summary>
    /// Called for each command processed by ActionService.
    /// </summary>
    public void OnCommand(Command command)
    {
      if (command.DisplayEffect != null)
      {
        RecordDisplayEffect(command.DisplayEffect);
      }

      if (command.FireProjectile != null)
      {
        RecordFireProjectile(command.FireProjectile);
      }

      if (command.DissolveCard != null)
      {
        RecordDissolveCard(command.DissolveCard);
      }

      if (command.SetCardTrail != null)
      {
        RecordSetCardTrail(command.SetCardTrail);
      }
    }

    public void ClearEffectLogs()
    {
      _entries.Clear();
    }

    public List<string>? TakeEffectLogs()
    {
      if (_entries.Count == 0)
      {
        return null;
      }

      var result = new List<string>(_entries);
      _entries.Clear();
      return result;
    }

    void RecordDisplayEffect(DisplayEffectCommand command)
    {
      var targetName = DescribeGameObjectId(command.Target);
      _entries.Add($"DisplayEffect: {command.Effect.Effect} on {targetName}");
    }

    void RecordFireProjectile(FireProjectileCommand command)
    {
      var sourceName = DescribeGameObjectId(command.SourceId);
      var targetName = DescribeGameObjectId(command.TargetId);
      _entries.Add(
        $"FireProjectile: {command.Projectile.Projectile} from {sourceName} to {targetName}"
      );
    }

    void RecordDissolveCard(DissolveCardCommand command)
    {
      var targetName = ResolveCardName(command.Target);
      _entries.Add($"DissolveCard: {targetName} (reverse={command.Reverse})");
    }

    void RecordSetCardTrail(SetCardTrailCommand command)
    {
      var count = command.CardIds.Count;
      _entries.Add(
        $"SetCardTrail: {command.Trail.Projectile} on {count} card{(count == 1 ? "" : "s")}"
      );
    }

    string DescribeGameObjectId(GameObjectId id)
    {
      if (id.CardId != null)
      {
        return ResolveCardName(id.CardId);
      }

      if (id.Deck != null)
      {
        return id.Deck == DisplayPlayer.User ? "UserDeck" : "EnemyDeck";
      }

      if (id.Void != null)
      {
        return id.Void == DisplayPlayer.User ? "UserVoid" : "EnemyVoid";
      }

      if (id.Avatar != null)
      {
        return id.Avatar == DisplayPlayer.User ? "UserAvatar" : "EnemyAvatar";
      }

      if (id.QuestObject != null)
      {
        return $"QuestObject({id.QuestObject})";
      }

      return "Unknown";
    }

    string ResolveCardName(string cardId)
    {
      var card = _cardService.GetCardIfExists(cardId);
      if (card != null)
      {
        var revealed = card.CardView.Revealed;
        if (revealed != null)
        {
          return DreamtidesSceneWalker.StripRichText(revealed.Name);
        }
      }

      return cardId;
    }
  }
}
