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

    public void ApplyLayouts(Sequence? sequence)
    {
      Registry.DreamscapeLayout.DraftPickLayout.ApplyLayout(sequence);
      Registry.DreamscapeLayout.ShopLayout.ApplyLayout(sequence);
      Registry.DreamscapeLayout.DreamsignDisplay.ApplyLayout(sequence);
      Registry.DreamscapeLayout.JourneyChoiceDisplay.ApplyLayout(sequence);
      Registry.DreamscapeLayout.TemptingOfferDisplay.ApplyLayout(sequence);

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

    public void HideShop()
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
      return _tmpSiteDeckLayout;
    }

    public ObjectLayout MerchantPositionLayout(Guid merchantId)
    {
      return _tmpMerchantPositionLayout;
    }

    public Transform CharacterScreenAnchorPosition(Guid merchantId)
    {
      return _tmpMerchantSpeechPosition;
    }
  }
}
