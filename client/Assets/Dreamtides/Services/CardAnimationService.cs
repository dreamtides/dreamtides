#nullable enable

using System;
using System.Collections;
using Dreamtides.Components;
using Dreamtides.Schema;
using Dreamtides.Animations;

namespace Dreamtides.Services
{
  public class CardAnimationService : Service
  {
    DrawUserCardsAnimation _drawUserCards = new();
    ShowAsDraftPickAnimation _showAsDraftPick = new();
    ShuffleVoidIntoDeckAnimation _shuffleVoidIntoDeck = new();
    InfoZoomAnimation _infoZoom = new();

    public bool IsPointerDownOnCard { get; set; } = false;

    protected override void OnInitialize(TestConfiguration? testConfiguration)
    {
      _infoZoom.Initialize(infoZoomDisabled: testConfiguration != null);
    }

    public IEnumerator HandleMoveCardsWithCustomAnimation(MoveCardsWithCustomAnimationCommand command)
    {
      switch (command.Animation)
      {
        case MoveCardsCustomAnimation.ShowAtDrawnCardsPosition:
          return _drawUserCards.Handle(command, this);
        case MoveCardsCustomAnimation.ShowInDraftPickLayout:
          return _showAsDraftPick.Handle(command, this);
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