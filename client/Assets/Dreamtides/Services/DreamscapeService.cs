#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using DG.Tweening;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Services
{
  public class DreamscapeService : Service
  {
    [SerializeField]
    ObjectLayout _tmpSiteDeckLayout = null!;

    [SerializeField]
    ObjectLayout _tmpMerchantPositionLayout = null!;

    [SerializeField]
    ObjectLayout _tmpTemptingOfferNpcLayout = null!;

    [SerializeField]
    Transform _tmpMerchantSpeechPosition = null!;

    [SerializeField]
    MecanimAnimator _tmpMerchantAnimator = null!;

    [SerializeField]
    CanvasGroup _closeButton = null!;

    [SerializeField]
    RectTransform _siteButtons = null!;

    [SerializeField]
    List<Dreamscape> _dreamscapes = new();

    [SerializeField]
    RectTransform _debugQuestButtons = null!;

    public CanvasGroup CloseButton => Errors.CheckNotNull(_closeButton);

    protected override void OnInitialize(GameMode mode, TestConfiguration? testConfiguration)
    {
      const int index = 0;
      if (mode == GameMode.Quest)
      {
        _dreamscapes[index].gameObject.SetActive(true);
        _siteButtons.gameObject.SetActive(true);
        _debugQuestButtons.gameObject.SetActive(true);
      }
      else
      {
        _dreamscapes[index].gameObject.SetActive(false);
        _siteButtons.gameObject.SetActive(false);
        _debugQuestButtons.gameObject.SetActive(false);
      }
    }

    public void HandleUpdateQuestCommand(
      UpdateQuestCommand command,
      List<Coroutine> coroutines,
      bool animate
    )
    {
      var essenceTotal = command.Quest.EssenceTotal > 0 ? command.Quest.EssenceTotal : 550;
      Registry.DreamscapeLayout.EssenceTotal.SetValue(essenceTotal.ToString(), true);
      Registry.DocumentService.RenderScreenOverlay(command.Quest.Interface?.ScreenOverlay);
      Registry.Layout.CardOrderSelector.View = command.Quest.Interface?.CardOrderSelector;
      Registry.Layout.UndoButton.SetView(command.Quest.Interface?.UndoButton);
      Registry.Layout.DevButton.SetView(command.Quest.Interface?.DevButton);
      Registry.Layout.CloseBrowserButton.CloseAction =
        command.Quest.Interface?.Browser?.CloseButton?.ToGameAction();
      Registry.DreamscapeLayout.TemptingOfferDisplay.SetOfferActions(
        command.Quest.TemptingOffer?.Actions
      );
      coroutines.Add(
        StartCoroutine(
          Registry.CardService.HandleUpdateQuestCards(
            command,
            animate ? TweenUtils.Sequence("UpdateQuest") : null
          )
        )
      );
    }

    public void ApplyLayouts(Sequence? sequence)
    {
      Registry.DreamscapeLayout.QuestDeck.ApplyLayout(sequence);
      Registry.DreamscapeLayout.QuestDeckBrowser.ApplyLayout(sequence);
      Registry.DreamscapeLayout.DraftPickLayout.ApplyLayout(sequence);
      Registry.DreamscapeLayout.DestroyedQuestCards.ApplyLayout(sequence);
      Registry.DreamscapeLayout.ShopLayout.ApplyLayout(sequence);
      Registry.DreamscapeLayout.DreamsignDisplay.ApplyLayout(sequence);
      Registry.DreamscapeLayout.JourneyChoiceDisplay.ApplyLayout(sequence);
      Registry.DreamscapeLayout.TemptingOfferDisplay.ApplyLayout(sequence);
      Registry.DreamscapeLayout.QuestEffectPosition.ApplyLayout(sequence);

      _tmpSiteDeckLayout.ApplyLayout(sequence);
      _tmpMerchantPositionLayout.ApplyLayout(sequence);
    }

    public void ShowShopWithCards(List<Card> cards)
    {
      foreach (var card in cards)
      {
        if (card.SpriteCardContentProtection)
        {
          card.SpriteCardContentProtection.gameObject.SetActive(true);
          TweenUtils.FadeInSprite(card.SpriteCardContentProtection);
        }
      }
    }

    public void HideCloseSiteButton()
    {
      TweenUtils
        .FadeOutCanvasGroup(_closeButton)
        .OnComplete(() =>
        {
          _closeButton.gameObject.SetActive(false);
        });
    }

    public IEnumerator HandlePlayMecanimAnimation(PlayMecanimAnimationCommand command)
    {
      return _tmpMerchantAnimator.PlayAnimation(command);
    }

    public ObjectLayout SiteDeckLayout(Guid siteId)
    {
      if (siteId == PrototypeQuest.DraftSiteId)
      {
        return _tmpSiteDeckLayout;
      }

      throw new InvalidOperationException($"Unknown site id: ${siteId}");
    }

    public ObjectLayout SiteNpcLayout(Guid siteId)
    {
      if (siteId == PrototypeQuest.ShopSiteId)
      {
        return _tmpMerchantPositionLayout;
      }

      if (siteId == PrototypeQuest.TemptingOfferSiteId)
      {
        return _tmpTemptingOfferNpcLayout;
      }

      throw new InvalidOperationException($"Unknown site id: ${siteId}");
    }

    public Transform CharacterScreenAnchorPosition(Guid merchantId)
    {
      return _tmpMerchantSpeechPosition;
    }
  }
}
