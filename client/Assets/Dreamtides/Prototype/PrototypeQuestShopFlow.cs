#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using Dreamtides.Masonry;
using Dreamtides.Prototype;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEngine;

public class PrototypeQuestShopFlow
{
  const string ShopGroupKey = "shop";
  const string HourglassPath =
    "Assets/ThirdParty/AngelinaAvgustova/WitchCraftIcons/PNG/outline_hourglass.png";

  readonly Registry _registry;
  readonly PrototypeCards _prototypeCards;
  readonly Func<CreateOrUpdateCardsRequest, bool, IEnumerator> _createOrUpdateCards;
  readonly Func<IEnumerator, Coroutine> _startCoroutine;
  readonly Guid _shopSiteId;
  List<string> _currentShopDisplayIds = new List<string>(6);
  List<CardOverride>? _shopOverrides;

  public PrototypeQuestShopFlow(
    Registry registry,
    PrototypeCards prototypeCards,
    Func<CreateOrUpdateCardsRequest, bool, IEnumerator> createOrUpdateCards,
    Func<IEnumerator, Coroutine> startCoroutine,
    Guid shopSiteId
  )
  {
    _registry = registry;
    _prototypeCards = prototypeCards;
    _createOrUpdateCards = createOrUpdateCards;
    _startCoroutine = startCoroutine;
    _shopSiteId = shopSiteId;
  }

  public void ConfigureShopOverrides(params CardOverride[] overrides)
  {
    _shopOverrides = overrides?.ToList();
  }

  public void ClearShopOverrides()
  {
    _shopOverrides = null;
  }

  public bool HasShopCard(string cardId) =>
    _currentShopDisplayIds.Count > 0 && _currentShopDisplayIds.Contains(cardId);

  public void ClearDisplayedCards()
  {
    _currentShopDisplayIds.Clear();
  }

  public IEnumerator PrepareShopCards()
  {
    _prototypeCards.ResetGroup(ShopGroupKey);
    EnsureShopOverrides();
    _shopOverrides!.RemoveAll(overrideData => overrideData.Index == 4);
    _shopOverrides.Add(
      new CardOverride { Index = 0, ButtonAttachmentLabel = "550<voffset=-0.17em>\ufcec</voffset>" }
    );
    _shopOverrides.Add(
      new CardOverride { Index = 1, ButtonAttachmentLabel = "200<voffset=-0.17em>\ufcec</voffset>" }
    );
    _shopOverrides.Add(
      new CardOverride { Index = 2, ButtonAttachmentLabel = "300<voffset=-0.17em>\ufcec</voffset>" }
    );
    _shopOverrides.Add(
      new CardOverride
      {
        Index = 3,
        Name = "Restock",
        SpritePath = "Assets/ThirdParty/GameAssets/Icons/outline_refresh-cw.png",
        Rules = "Restock",
        Prefab = CardPrefab.IconCard,
        CardType = "Restock",
        ButtonAttachmentLabel = "100<voffset=-0.17em>\ufcec</voffset>",
      }
    );
    _shopOverrides.Add(
      new CardOverride
      {
        Index = 4,
        SpritePath = HourglassPath,
        Prefab = CardPrefab.Dreamsign,
        ButtonAttachmentLabel = "200<voffset=-0.17em>\ufcec</voffset>",
      }
    );
    _shopOverrides.Add(
      new CardOverride
      {
        Index = 5,
        SpritePath =
          "Assets/ThirdParty/AngelinaAvgustova/WitchCraftIcons/PNG/outline_goldfeather.png",
        Prefab = CardPrefab.Dreamsign,
        ButtonAttachmentLabel = "600<voffset=-0.17em>\ufcec</voffset>",
      }
    );

    var request = new CreateOrUpdateCardsRequest
    {
      Count = 6,
      Position = new ObjectPosition
      {
        Position = new Position { PositionClass = new PositionClass { SiteNpc = _shopSiteId } },
        SortingKey = 1,
      },
      Revealed = true,
      GroupKey = ShopGroupKey,
      Overrides = _shopOverrides,
      ButtonAttachmentDebugScenario = "shop-pick",
    };
    yield return _createOrUpdateCards(request, false);
  }

  public IEnumerator RunShopDisplaySequence()
  {
    EnsureShopOverrides();
    var idx4 = _shopOverrides!.FindIndex(overrideData => overrideData.Index == 4);
    if (idx4 >= 0)
    {
      _shopOverrides[idx4].SpritePath = HourglassPath;
    }
    else
    {
      _shopOverrides.Add(
        new CardOverride
        {
          Index = 4,
          SpritePath = HourglassPath,
          ButtonAttachmentLabel = "200<voffset=-0.17em>\ufcec</voffset>",
        }
      );
    }

    var request = new CreateOrUpdateCardsRequest
    {
      Count = 6,
      Position = new ObjectPosition
      {
        Position = new Position { Enum = PositionEnum.ShopDisplay },
        SortingKey = 1,
      },
      Revealed = true,
      GroupKey = ShopGroupKey,
      Overrides = _shopOverrides,
      ButtonAttachmentLabel = null,
      ButtonAttachmentDebugScenario = "shop-pick",
    };

    var allCards = _prototypeCards.CreateOrUpdateCards(request);
    _currentShopDisplayIds = allCards.Take(6).Select(card => card.Id).ToList();

    yield return new WaitForSeconds(0.3f);

    var customAnimation = new MoveCardsWithCustomAnimationCommand
    {
      Animation = MoveCardsCustomAnimation.ShowInShopLayout,
      Cards = allCards.Take(6).ToList(),
      Destination = new Position { Enum = PositionEnum.ShopDisplay },
      PauseDuration = new Milliseconds { MillisecondsValue = 0 },
      StaggerInterval = new Milliseconds { MillisecondsValue = 50 },
    };

    _startCoroutine(
      _registry.DreamscapeService.HandlePlayMecanimAnimation(
        new PlayMecanimAnimationCommand
        {
          SiteId = _shopSiteId,
          Parameters = new List<MecanimParameter>
          {
            new MecanimParameter { TriggerParam = new TriggerParam { Name = "WaveSmall" } },
          },
        }
      )
    );

    yield return _registry.CardAnimationService.HandleMoveCardsWithCustomAnimation(customAnimation);

    _registry.DocumentService.RenderScreenAnchoredNode(
      new AnchorToScreenPositionCommand()
      {
        Node = ShopMerchantDialog(),
        Anchor = new ScreenAnchor { SiteCharacter = _shopSiteId },
        ShowDuration = new Milliseconds { MillisecondsValue = 5000 },
      }
    );

    yield return _createOrUpdateCards(request, true);

    var button = _registry.DreamscapeService.CloseSiteButton.GetComponent<CloseBrowserButton>();
    button.CloseAction = new GameAction
    {
      GameActionClass = new GameActionClass
      {
        DebugAction = new DebugAction
        {
          DebugActionClass = new DebugActionClass { ApplyTestScenarioAction = "closeShop" },
        },
      },
    };
  }

  public IEnumerator ResolveShopPick(string clickedId)
  {
    var cardsForAnimation = new List<CardView>(1);
    var card = _registry.CardService.GetCard(clickedId);
    var source = card.CardView;
    var isRestock = source.Revealed != null && source.Revealed.CardType == "Restock";
    if (isRestock)
    {
      yield return RestockShopSequence();
      yield break;
    }
    var isDreamsign = source.Prefab == CardPrefab.Dreamsign;
    if (isDreamsign)
    {
      var sorting = _registry.DreamscapeLayout.DreamsignDisplay.Objects.Count;
      cardsForAnimation.Add(
        PrototypeQuestCardViewFactory.CloneCardViewWithPosition(
          source,
          new Position { Enum = PositionEnum.DreamsignDisplay },
          sorting
        )
      );
    }
    else
    {
      var sorting = _registry.DreamscapeLayout.QuestDeck.Objects.Count;
      cardsForAnimation.Add(
        PrototypeQuestCardViewFactory.CloneCardViewWithPosition(
          source,
          new Position { Enum = PositionEnum.QuestDeck },
          sorting
        )
      );
    }

    var command = new MoveCardsWithCustomAnimationCommand
    {
      Animation = isDreamsign
        ? MoveCardsCustomAnimation.MoveToDreamsignDisplayOrDestroy
        : MoveCardsCustomAnimation.MoveToQuestDeckOrDestroy,
      Cards = cardsForAnimation,
      Destination = new Position
      {
        Enum = isDreamsign ? PositionEnum.DreamsignDisplay : PositionEnum.QuestDeck,
      },
      PauseDuration = new Milliseconds { MillisecondsValue = 300 },
      StaggerInterval = new Milliseconds { MillisecondsValue = 100 },
      CardTrail = new ProjectileAddress
      {
        Projectile =
          "Assets/ThirdParty/Hovl Studio/AAA Projectiles Vol 1/Prefabs/Dreamtides/Projectile 26 blue diamond.prefab",
      },
    };

    yield return _registry.CardAnimationService.HandleMoveCardsWithCustomAnimation(command);

    var allIds = _registry.CardService.GetCardIds().ToList();
    var updateCards = new List<CardView>(allIds.Count);
    foreach (var id in allIds)
    {
      var current = _registry.CardService.GetCard(id);
      var view = current.CardView;
      if (id == clickedId)
      {
        if (isDreamsign)
        {
          var sorting = _registry.DreamscapeLayout.DreamsignDisplay.Objects.Count;
          var clone = PrototypeQuestCardViewFactory.CloneCardViewWithPosition(
            view,
            new Position { Enum = PositionEnum.DreamsignDisplay },
            sorting
          );
          if (clone.Revealed != null && clone.Revealed.Actions != null)
          {
            clone.Revealed.Actions.ButtonAttachment = null;
          }
          updateCards.Add(clone);
        }
        else
        {
          var sorting = _registry.DreamscapeLayout.QuestDeck.Objects.Count;
          updateCards.Add(
            PrototypeQuestCardViewFactory.CloneCardViewWithPositionHidden(
              view,
              new Position { Enum = PositionEnum.QuestDeck },
              sorting
            )
          );
        }
      }
      else
      {
        updateCards.Add(view);
      }
    }

    var update = new UpdateQuestCommand { Quest = new QuestView { Cards = updateCards } };
    var sequence = TweenUtils.Sequence("UpdateQuest");
    yield return _registry.CardService.HandleUpdateQuestCards(update, sequence);

    _prototypeCards.UpdateGroupCards(ShopGroupKey, updateCards);
    _currentShopDisplayIds.Remove(clickedId);
  }

  public IEnumerator RestockShopSequence()
  {
    yield return HideShopSequence();

    var allIds = _registry.CardService.GetCardIds().ToList();
    var updateCards = new List<CardView>(allIds.Count);
    foreach (var id in allIds)
    {
      var current = _registry.CardService.GetCard(id);
      var view = current.CardView;
      if (_currentShopDisplayIds.Contains(id))
      {
        updateCards.Add(
          PrototypeQuestCardViewFactory.CloneCardViewWithPositionHidden(
            view,
            new Position { Enum = PositionEnum.Offscreen },
            0
          )
        );
      }
      else
      {
        updateCards.Add(view);
      }
    }

    var update = new UpdateQuestCommand { Quest = new QuestView { Cards = updateCards } };
    yield return _registry.CardService.HandleUpdateQuestCards(update);

    _prototypeCards.UpdateGroupCards(ShopGroupKey, updateCards);
    _prototypeCards.AdvanceGroupWindow(ShopGroupKey, 6);

    yield return RunShopDisplaySequence();
  }

  IEnumerator HideShopSequence()
  {
    var cardsForAnimation = new List<CardView>(_currentShopDisplayIds.Count);
    if (_currentShopDisplayIds.Count > 0)
    {
      foreach (var id in _currentShopDisplayIds)
      {
        var card = _registry.CardService.GetCard(id);
        cardsForAnimation.Add(card.CardView);
      }
    }
    else
    {
      foreach (var displayable in _registry.DreamscapeLayout.ShopLayout.Objects)
      {
        if (displayable is Dreamtides.Components.Card card)
        {
          cardsForAnimation.Add(card.CardView);
        }
      }
    }

    var command = new MoveCardsWithCustomAnimationCommand
    {
      Animation = MoveCardsCustomAnimation.HideShopLayout,
      Cards = cardsForAnimation,
      Destination = new Position { Enum = PositionEnum.ShopDisplay },
      PauseDuration = new Milliseconds { MillisecondsValue = 0 },
      StaggerInterval = new Milliseconds { MillisecondsValue = 50 },
    };

    yield return _registry.CardAnimationService.HandleMoveCardsWithCustomAnimation(command);
  }

  static FlexNode ShopMerchantDialog()
  {
    var style = new FlexStyle
    {
      BackgroundColor = Mason.MakeColor(Color.black),
      BorderRadius = Mason.AllBordersRadiusDip(4),
      Padding = Mason.AllPx(4),
      Inset = Mason.InsetPx(top: -8f, left: 6f),
      Color = Mason.MakeColor(Color.white),
      Position = FlexPosition.Absolute,
      FontSize = Mason.Px(8),
      Font = new FontAddress
      {
        Font = "Assets/ThirdParty/Fonts/Fira_Sans_Condensed/FiraSansCondensed-Regular.ttf",
      },
      TextAlign = TextAlign.MiddleLeft,
      AlignItems = FlexAlign.Center,
      JustifyContent = FlexJustify.Center,
      WhiteSpace = WhiteSpace.Normal,
      MaxWidth = Mason.Px(100),
    };

    var arrowTip = Mason.Column(
      "ArrowPoint",
      new FlexStyle
      {
        Width = Mason.Px(9),
        Height = Mason.Px(11.5f),
        Position = FlexPosition.Absolute,
        Inset = Mason.InsetPx(top: 0f, left: 0f),
        AlignItems = FlexAlign.Center,
        JustifyContent = FlexJustify.Center,
        BackgroundImage = new SpriteAddress
        {
          Sprite = "Assets/ThirdParty/GameAssets/speech_bubble_tip.png",
        },
      }
    );

    var text = Mason.TypewriterText(
      "Cooked up some nice grub for ya!",
      style,
      30,
      new AudioClipAddress
      {
        AudioClip = "Assets/ThirdParty/Cafofo/Fantasy Interface Sounds/UI Tight 01.wav",
      }
    );

    return Mason.Row(
      "ShopMerchantDialog",
      new FlexStyle { AlignItems = FlexAlign.FlexEnd, JustifyContent = FlexJustify.FlexStart },
      arrowTip,
      text
    );
  }

  void EnsureShopOverrides()
  {
    if (_shopOverrides == null)
    {
      _shopOverrides = new List<CardOverride>();
    }
  }
}
