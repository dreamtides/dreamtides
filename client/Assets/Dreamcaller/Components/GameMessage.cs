#nullable enable

using System;
using System.Collections;
using DG.Tweening;
using Dreamcaller.Schema;
using Dreamcaller.Services;
using Dreamcaller.Utils;
using TMPro;
using UnityEngine;

namespace Dreamcaller.Components
{
  [Serializable]
  public class MessageContent
  {
    [SerializeField] GameObject _effect = null!;
    public GameObject Effect => _effect;

    [SerializeField] TextMeshPro _text = null!;
    public TextMeshPro Text => _text;
  }

  public class GameMessage : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] Transform _top = null!;
    [SerializeField] MessageContent _yourTurn = null!;
    [SerializeField] MessageContent _enemyTurn = null!;
    [SerializeField] MessageContent _victory = null!;
    [SerializeField] MessageContent _defeat = null!;

    IEnumerator Start()
    {
      yield return new WaitForSeconds(3f);
      yield return Show(GameMessageType.YourTurn);
    }

    public IEnumerator Show(GameMessageType messageType)
    {
      _registry.SoundService.PlayMessageSound(messageType);
      switch (messageType)
      {
        case GameMessageType.Victory:
          _registry.MusicService.Mute();
          break;
        case GameMessageType.Defeat:
          _registry.MusicService.Mute();
          break;
      }

      return messageType switch
      {
        GameMessageType.YourTurn => ShowContent(_yourTurn, 1.75f, moveToTop: false),
        GameMessageType.EnemyTurn => ShowContent(_enemyTurn, 1.75f, moveToTop: false),
        GameMessageType.Victory => ShowContent(_victory, 2f, moveToTop: true),
        GameMessageType.Defeat => ShowContent(_defeat, 2f, moveToTop: true),
        _ => throw Errors.UnknownEnumValue(messageType)
      };
    }

    IEnumerator ShowContent(MessageContent content, float durationSeconds, bool moveToTop)
    {
      content.Effect.gameObject.SetActive(true);
      content.Text.gameObject.SetActive(true);
      content.Text.alpha = 0f;
      yield return DOTween
        .To(() => content.Text.alpha, x => content.Text.alpha = x, endValue: 1f, 0.2f)
        .WaitForCompletion();
      yield return new WaitForSeconds(durationSeconds);

      if (moveToTop)
      {
        _registry.BattlefieldOverlay.gameObject.SetActive(true);
        _registry.BattlefieldOverlay.color = new Color(0, 0, 0, 0);
        _registry.BattlefieldOverlay.DOFade(1.0f, 0.3f);
        var sequence = TweenUtils.Sequence("MoveToTop")
          .Insert(0, content.Text.transform.DOMove(_top.position, 0.3f));
        sequence.Insert(0, content.Effect.transform.DOMove(_top.position, 0.3f));
        yield return sequence.WaitForCompletion();
      }
      else
      {
        yield return DOTween
          .To(() => content.Text.alpha, x => content.Text.alpha = x, endValue: 0f, 0.2f)
          .WaitForCompletion();
        content.Text.gameObject.SetActive(false);
      }
    }
  }
}
