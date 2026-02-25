#nullable enable

using Abu;
using Dreamtides.Buttons;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Sites;
using UnityEngine;
using Object = UnityEngine.Object;

namespace Dreamtides.Abu
{
  public partial class DreamtidesSceneWalker
  {
    // ── Quest mode walk ──────────────────────────────────────────────

    AbuSceneNode WalkQuest(RefRegistry refRegistry)
    {
      var region = CreateRegionNode("Quest");
      var layout = _registry.DreamscapeLayout;
      var hasOpenPanels = _registry.DocumentService.HasOpenPanels;

      // 1. Controls (always shown)
      region.Children.Add(WalkQuestControls(refRegistry));

      if (!hasOpenPanels)
      {
        // 2. Map site buttons (when visible)
        var mapGroup = WalkMapSiteButtons(refRegistry);
        if (mapGroup != null)
        {
          region.Children.Add(mapGroup);
        }

        // 3. Essence
        AddEssenceLabel(region);

        // 4. Quest Deck summary + browse button
        AddQuestDeckSummary(region, layout, refRegistry);

        // 5. Identity card
        var identityGroup = WalkQuestCardGroup(
          "Identity",
          PositionEnum.QuestUserIdentityCard,
          refRegistry
        );
        if (identityGroup != null)
        {
          region.Children.Add(identityGroup);
        }

        // 6. Dreamsigns
        var dreamsignsGroup = WalkDreamsigns(layout, refRegistry);
        if (dreamsignsGroup != null)
        {
          region.Children.Add(dreamsignsGroup);
        }

        // 7. Draft picks
        var draftGroup = WalkQuestCardGroup(
          "Draft Picks",
          PositionEnum.DraftPickDisplay,
          refRegistry
        );
        if (draftGroup != null)
        {
          region.Children.Add(draftGroup);
        }

        // 8. Shop
        var shopGroup = WalkQuestCardGroup("Shop", PositionEnum.ShopDisplay, refRegistry);
        if (shopGroup != null)
        {
          region.Children.Add(shopGroup);
        }

        // 9. Tempting offer (with accept buttons)
        var offerGroup = WalkTemptingOffer(layout, refRegistry);
        if (offerGroup != null)
        {
          region.Children.Add(offerGroup);
        }

        // 10. Start battle (with start button)
        var battleGroup = WalkStartBattle(layout, refRegistry);
        if (battleGroup != null)
        {
          region.Children.Add(battleGroup);
        }

        // 11. Journey choices
        var journeyGroup = WalkQuestCardGroup(
          "Journey Choices",
          PositionEnum.JourneyDisplay,
          refRegistry
        );
        if (journeyGroup != null)
        {
          region.Children.Add(journeyGroup);
        }

        // 12. Quest deck browser
        var browserGroup = WalkQuestCardGroup(
          "Quest Deck Browser",
          PositionEnum.QuestDeckBrowser,
          refRegistry
        );
        if (browserGroup != null)
        {
          region.Children.Add(browserGroup);
        }
      }

      // 13. Card order selector (reuse from battle walker)
      var cardOrderGroup = WalkCardOrderSelector(_registry.BattleLayout, refRegistry);
      if (cardOrderGroup != null)
      {
        region.Children.Add(cardOrderGroup);
      }

      // 14. UIToolkit overlays (filtered)
      var uiOverlay = WalkUiToolkitFiltered(refRegistry);
      if (uiOverlay != null)
      {
        region.Children.Add(uiOverlay);
      }

      return region;
    }

    // ── Quest controls ───────────────────────────────────────────────

    AbuSceneNode WalkQuestControls(RefRegistry refRegistry)
    {
      var group = CreateGroupNode("Controls");
      var doc = _registry.DocumentService;

      TryAddCanvasButton(group, refRegistry, doc.MenuButton, "Menu");
      TryAddCanvasButton(group, refRegistry, doc.UndoButton, "Undo");
      TryAddCanvasButton(group, refRegistry, doc.DevButton, "Dev");
      TryAddCanvasButton(group, refRegistry, doc.BugButton, "Bug Report");

      // Quest deck browser close button
      TryAddCloseButton(
        group,
        refRegistry,
        _registry.DreamscapeLayout.QuestDeckBrowser._closeButton,
        "Close Browser"
      );

      // Site close button (on DreamscapeService CanvasGroup)
      TryAddCloseSiteButton(group, refRegistry);

      return group;
    }

    // ── Close button helper (shared) ─────────────────────────────────

    void TryAddCloseButton(
      AbuSceneNode parent,
      RefRegistry refRegistry,
      CloseBrowserButton? button,
      string label
    )
    {
      if (button == null || !button.gameObject.activeSelf)
      {
        return;
      }

      AddInteractiveNode(
        parent,
        refRegistry,
        "button",
        label,
        new RefCallbacks { OnClick = () => button.OnClick() }
      );
    }

    void TryAddCloseSiteButton(AbuSceneNode parent, RefRegistry refRegistry)
    {
      var dreamscapeService = _registry._dreamscapeService;
      if (dreamscapeService == null)
      {
        return;
      }

      var canvasGroup = dreamscapeService._closeSiteButton;
      if (canvasGroup == null || !canvasGroup.gameObject.activeSelf || canvasGroup.alpha <= 0)
      {
        return;
      }

      var button = canvasGroup.GetComponent<CloseBrowserButton>();
      if (button == null)
      {
        return;
      }

      AddInteractiveNode(
        parent,
        refRegistry,
        "button",
        "Close Site",
        new RefCallbacks { OnClick = () => button.OnClick() }
      );
    }

    // ── Map site buttons ─────────────────────────────────────────────

    AbuSceneNode? WalkMapSiteButtons(RefRegistry refRegistry)
    {
      var camera = Object.FindFirstObjectByType<DreamscapeMapCamera>(FindObjectsInactive.Exclude);
      if (camera == null)
      {
        return null;
      }

      var group = CreateGroupNode("Map");
      foreach (var kvp in camera._siteButtonsBySite)
      {
        var label = SiteLabel(kvp.Key);
        TryAddCanvasButton(group, refRegistry, kvp.Value, label);
      }

      return group.Children.Count > 0 ? group : null;
    }

    static string SiteLabel(AbstractDreamscapeSite site)
    {
      return site.DebugClickAction switch
      {
        "FocusDraftCamera" => "Draft",
        "FocusShopCamera" => "Shop",
        "FocusEventCamera" => "Event",
        "FocusEssenceCamera" => "Essence",
        "FocusBattleCamera" => "Battle",
        _ => site.GetType().Name,
      };
    }

    // ── Quest deck summary ───────────────────────────────────────────

    void AddQuestDeckSummary(AbuSceneNode parent, DreamscapeLayout layout, RefRegistry refRegistry)
    {
      var deckCount = layout.QuestDeck.Objects.Count;
      if (deckCount == 0)
      {
        return;
      }

      var group = CreateGroupNode($"Quest Deck ({deckCount} cards)");

      // Find the QuestDeck CardBrowserButton to use for browse action
      foreach (
        var btn in Object.FindObjectsByType<CardBrowserButton>(
          FindObjectsInactive.Exclude,
          FindObjectsSortMode.None
        )
      )
      {
        if (btn._type == CardBrowserType.QuestDeck && btn.CanHandleMouseEvents())
        {
          AddInteractiveNode(
            group,
            refRegistry,
            "button",
            "Browse Quest Deck",
            BuildDisplayableCallbacks(btn)
          );
          break;
        }
      }

      parent.Children.Add(group);
    }

    // ── Dreamsigns ───────────────────────────────────────────────────

    AbuSceneNode? WalkDreamsigns(DreamscapeLayout layout, RefRegistry refRegistry)
    {
      var dreamsignLayout = layout.DreamsignDisplay;
      if (dreamsignLayout.Objects.Count == 0)
      {
        return null;
      }

      var group = CreateGroupNode("Dreamsigns");
      foreach (var obj in dreamsignLayout.Objects)
      {
        if (!obj.CanHandleMouseEvents())
        {
          continue;
        }

        var label = DetermineDisplayableLabel(obj);
        AddInteractiveNode(group, refRegistry, "button", label, BuildDisplayableCallbacks(obj));
      }

      return group.Children.Count > 0 ? group : null;
    }

    // ── Tempting offer ───────────────────────────────────────────────

    AbuSceneNode? WalkTemptingOffer(DreamscapeLayout layout, RefRegistry refRegistry)
    {
      var offerLayout = layout.TemptingOfferDisplay;
      if (offerLayout.Objects.Count == 0)
      {
        return null;
      }

      var group = CreateGroupNode("Tempting Offer");
      AddCardNodes(group, offerLayout.Objects, "Browser", refRegistry);

      // Walk accept buttons
      foreach (var button in offerLayout._acceptButtons)
      {
        if (button == null || !button.gameObject.activeSelf)
        {
          continue;
        }

        var label = ToSingleLineText(button._text.text, fallback: "Accept");
        AddInteractiveNode(group, refRegistry, "button", label, BuildDisplayableCallbacks(button));
      }

      return group.Children.Count > 0 ? group : null;
    }

    // ── Start battle ─────────────────────────────────────────────────

    AbuSceneNode? WalkStartBattle(DreamscapeLayout layout, RefRegistry refRegistry)
    {
      var startLayout = layout.StartBattleLayout;
      if (startLayout.Objects.Count == 0)
      {
        return null;
      }

      var group = CreateGroupNode("Start Battle");
      AddCardNodes(group, startLayout.Objects, "Browser", refRegistry);

      // Walk the "Start Battle" button
      if (startLayout._buttonInstance != null && startLayout._buttonInstance.gameObject.activeSelf)
      {
        var label = ToSingleLineText(
          startLayout._buttonInstance._text.text,
          fallback: "Start Battle"
        );
        AddInteractiveNode(
          group,
          refRegistry,
          "button",
          label,
          BuildDisplayableCallbacks(startLayout._buttonInstance)
        );
      }

      return group.Children.Count > 0 ? group : null;
    }

    // ── Generic quest card group ─────────────────────────────────────

    /// <summary>
    /// Finds cards at a given position by scanning all active Card objects.
    /// Animations may remove cards from their parent layout while keeping
    /// them active in the scene, so layout.Objects can be empty even when
    /// cards are visible. This method finds cards by their ObjectPosition
    /// instead.
    /// </summary>
    AbuSceneNode? WalkQuestCardGroup(string label, PositionEnum position, RefRegistry refRegistry)
    {
      var group = CreateGroupNode(label);
      foreach (
        var card in Object.FindObjectsByType<Card>(
          FindObjectsInactive.Exclude,
          FindObjectsSortMode.None
        )
      )
      {
        if (card.ObjectPosition?.Position.Enum != position)
        {
          continue;
        }

        var cardNode = BuildCardNode(card, "Browser", refRegistry);
        if (cardNode != null)
        {
          group.Children.Add(cardNode);
        }

        // Walk button attachment (e.g. shop purchase buttons)
        var buttonAttachment = card.ButtonAttachment;
        if (buttonAttachment != null && buttonAttachment.gameObject.activeSelf)
        {
          var buttonLabel = ToSingleLineText(buttonAttachment._text.text, fallback: "Buy");
          var cardName =
            card.CardView.Revealed != null
              ? ToSingleLineText(card.CardView.Revealed.Name, fallback: "card")
              : "card";
          AddInteractiveNode(
            group,
            refRegistry,
            "button",
            $"Buy {cardName} (cost: {buttonLabel})",
            BuildDisplayableCallbacks(buttonAttachment)
          );
        }
      }

      return group.Children.Count > 0 ? group : null;
    }
  }
}
