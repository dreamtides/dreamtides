#nullable enable

using System;
using System.Collections;
using Dreamtides.Animations;
using Dreamtides.Components;
using Dreamtides.Schema;
using UnityEngine;

namespace Dreamtides.Services
{
  public class CardAnimationService : Service
  {
    [SerializeField]
    AudioClip _flipCardSound = null!;
    public AudioClip FlipCardSound => _flipCardSound;

    [SerializeField]
    AudioClip _moveToQuestDeckSound = null!;
    public AudioClip MoveToQuestDeckSound => _moveToQuestDeckSound;

    MoveCardsDefaultAnimation _moveCardsDefault = new();
    DrawUserCardsAnimation _drawUserCards = new();
    ShowAsDraftPickAnimation _showAsDraftPick = new();
    ShowInShopLayoutAnimation _showInShopLayout = new();
    ShuffleVoidIntoDeckAnimation _shuffleVoidIntoDeck = new();
    InfoZoomAnimation _infoZoom = new();
    MoveToQuestDeckOrDestroyAnimation _moveToQuestDeckOrDestroy = new();
    MoveToDreamsignDisplayOrDestroyAnimation _moveToDreamsignDisplayOrDestroy = new();
    OpenQuestDeckBrowserAnimation _openQuestDeckBrowser = new();

    public bool IsPointerDownOnCard { get; set; } = false;

    protected override void OnInitialize(GameMode _mode, TestConfiguration? testConfiguration)
    {
      _infoZoom.Initialize(infoZoomDisabled: testConfiguration != null);
    }

    public IEnumerator HandleMoveCardsWithCustomAnimation(
      MoveCardsWithCustomAnimationCommand command
    )
    {
      switch (command.Animation)
      {
        case MoveCardsCustomAnimation.DefaultAnimation:
          return _moveCardsDefault.Handle(command, this);
        case MoveCardsCustomAnimation.ShowAtDrawnCardsPosition:
          return _drawUserCards.Handle(command, this);
        case MoveCardsCustomAnimation.ShowInDraftPickLayout:
          return _showAsDraftPick.Handle(command, this);
        case MoveCardsCustomAnimation.ShowInShopLayout:
          return _showInShopLayout.Handle(command, this);
        case MoveCardsCustomAnimation.HideShopLayout:
          return _showInShopLayout.HandleHide(command, this);
        case MoveCardsCustomAnimation.MoveToQuestDeckOrDestroy:
          return _moveToQuestDeckOrDestroy.Handle(command, this);
        case MoveCardsCustomAnimation.MoveToDreamsignDisplayOrDestroy:
          return _moveToDreamsignDisplayOrDestroy.Handle(command, this);
        case MoveCardsCustomAnimation.OpenQuestDeckBrowser:
          return _openQuestDeckBrowser.Handle(command, this);
        default:
          throw new IndexOutOfRangeException($"Unhandled animation type: {command.Animation}");
      }
    }

    public IEnumerator HandleShuffleVoidIntoDeck(ShuffleVoidIntoDeckCommand command)
    {
      return _shuffleVoidIntoDeck.Handle(command, this);
    }

    /// <summary>
    /// Displays a large format version of the provided card in the info zoom.
    /// </summary>
    public void DisplayInfoZoom(Card card, bool forCardInHand)
    {
      _infoZoom.DisplayInfoZoom(this, card, forCardInHand);
    }

    public void ClearInfoZoom()
    {
      _infoZoom.ClearInfoZoom(this);
    }
  }
}
