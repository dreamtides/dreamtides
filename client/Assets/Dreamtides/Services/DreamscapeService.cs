#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using DG.Tweening;
using Dreamtides.Buttons;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Sites;
using Dreamtides.Utils;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Services
{
  public class DreamscapeService : Service
  {
    [SerializeField]
    internal CanvasGroup _closeSiteButton = null!;

    [SerializeField]
    internal CanvasButton _siteButtonPrefab = null!;

    [SerializeField]
    internal RectTransform _siteButtons = null!;

    [SerializeField]
    List<Dreamscape> _dreamscapes = new();

    [SerializeField]
    internal RectTransform _debugQuestButtons = null!;

    public CanvasGroup CloseSiteButton => Errors.CheckNotNull(_closeSiteButton);

    public CanvasButton CreateOpenSiteButton()
    {
      var button = ComponentUtils.Instantiate(_siteButtonPrefab, Vector3.zero);
      var rectTransform = ComponentUtils.Get<RectTransform>(button);
      rectTransform.SetParent(Registry.CanvasSafeArea, false);
      return button;
    }

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
      Registry.BattleLayout.CardOrderSelector.View = command.Quest.Interface?.CardOrderSelector;
      Registry.BattleLayout.UndoButton.SetView(
        new ButtonView
        {
          Action = new OnClickClass
          {
            DebugAction = new DebugAction
            {
              DebugActionClass = new DebugActionClass
              {
                ApplyTestScenarioAction = "FocusMapCamera",
              },
            },
          },
          Label = "U",
        }
      );
      Registry.BattleLayout.DevButton.SetView(command.Quest.Interface?.DevButton);
      Registry.BattleLayout.CloseBrowserButton.CloseAction =
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
      Registry.DreamscapeLayout.StartBattleLayout.ApplyLayout(sequence);
      ApplySiteOwnedLayouts(sequence);
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
        .FadeOutCanvasGroup(_closeSiteButton)
        .OnComplete(() =>
        {
          _closeSiteButton.gameObject.SetActive(false);
        });
    }

    public IEnumerator HandlePlayMecanimAnimation(PlayMecanimAnimationCommand command)
    {
      var site = FindCharacterSite(command.SiteId);
      var animator = Errors.CheckNotNull(site.CharacterAnimator);
      return animator.PlayAnimation(command);
    }

    public ObjectLayout SiteDeckLayout(Guid siteId)
    {
      var site = FindDraftSite(siteId);
      return site.SiteDeckLayout;
    }

    public ObjectLayout SiteCharacterOwnedLayout(Guid siteId)
    {
      var site = FindCharacterSite(siteId);
      return Errors.CheckNotNull(site.CharacterOwnedObjects);
    }

    public Transform CharacterScreenAnchorPosition(Guid merchantId)
    {
      var site = FindCharacterSite(merchantId);
      return Errors.CheckNotNull(site.CharacterSpeechPosition);
    }

    CharacterSite FindCharacterSite(Guid siteId)
    {
      var sites = FindObjectsByType<CharacterSite>(
        FindObjectsInactive.Exclude,
        FindObjectsSortMode.None
      );
      for (var i = 0; i < sites.Length; i++)
      {
        var site = sites[i];
        if (site != null && site.SiteId == siteId)
        {
          return site;
        }
      }
      throw new InvalidOperationException($"Unknown character site id: {siteId}");
    }

    DraftSite FindDraftSite(Guid siteId)
    {
      var sites = FindObjectsByType<DraftSite>(
        FindObjectsInactive.Exclude,
        FindObjectsSortMode.None
      );
      for (var i = 0; i < sites.Length; i++)
      {
        var site = sites[i];
        if (site != null && site.SiteId == siteId)
        {
          return site;
        }
      }
      throw new InvalidOperationException($"Unknown draft site id: {siteId}");
    }

    void ApplySiteOwnedLayouts(Sequence? sequence)
    {
      var sites = FindObjectsByType<CharacterSite>(
        FindObjectsInactive.Exclude,
        FindObjectsSortMode.None
      );
      for (var i = 0; i < sites.Length; i++)
      {
        var site = sites[i];
        var layout = site?.CharacterOwnedObjects;
        if (layout != null)
        {
          layout.ApplyLayout(sequence);
        }
      }
    }
  }
}
