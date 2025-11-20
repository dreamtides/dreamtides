#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using Dreamtides.Prototype;
using Dreamtides.Schema;
using Dreamtides.Services;

public class PrototypeQuestDraftFlow
{
  readonly Registry _registry;
  readonly PrototypeCards _prototypeCards;
  readonly Func<CreateOrUpdateCardsRequest, bool, IEnumerator> _createOrUpdateCards;
  readonly Action _focusMapCamera;
  readonly Func<string> _outlineColorProvider;
  readonly Guid _draftSiteId;
  List<string> _currentDraftPickIds = new List<string>(4);

  public PrototypeQuestDraftFlow(
    Registry registry,
    PrototypeCards prototypeCards,
    Func<CreateOrUpdateCardsRequest, bool, IEnumerator> createOrUpdateCards,
    Action focusMapCamera,
    Func<string> outlineColorProvider,
    Guid draftSiteId
  )
  {
    _registry = registry;
    _prototypeCards = prototypeCards;
    _createOrUpdateCards = createOrUpdateCards;
    _focusMapCamera = focusMapCamera;
    _outlineColorProvider = outlineColorProvider;
    _draftSiteId = draftSiteId;
  }

  public IEnumerator PrepareDraftDeck()
  {
    var request = new CreateOrUpdateCardsRequest
    {
      Count = 20,
      Position = new ObjectPosition
      {
        Position = new Position { PositionClass = new PositionClass { SiteDeck = _draftSiteId } },
        SortingKey = 1,
      },
      Revealed = false,
      GroupKey = "draft",
    };
    yield return _createOrUpdateCards(request, true);
  }

  public bool HasDraftPick(string cardId) =>
    _currentDraftPickIds.Count == 4 && _currentDraftPickIds.Contains(cardId);

  public IEnumerator ResolveDraftPick(string clickedId)
  {
    var cardsForAnimation = new List<CardView>(4);
    foreach (var id in _currentDraftPickIds)
    {
      var card = _registry.CardService.GetCard(id);
      var source = card.CardView;
      if (id == clickedId)
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
      else
      {
        cardsForAnimation.Add(source);
      }
    }

    var command = new MoveCardsWithCustomAnimationCommand
    {
      Animation = MoveCardsCustomAnimation.MoveToQuestDeckOrDestroy,
      Cards = cardsForAnimation,
      Destination = new Position { Enum = PositionEnum.QuestDeck },
      PauseDuration = new Milliseconds { MillisecondsValue = 0 },
      StaggerInterval = new Milliseconds { MillisecondsValue = 0 },
    };

    yield return _registry.CardAnimationService.HandleMoveCardsWithCustomAnimation(command);

    var allIds = _registry.CardService.GetCardIds().ToList();
    var updateCards = new List<CardView>(allIds.Count);
    foreach (var id in allIds)
    {
      var card = _registry.CardService.GetCard(id);
      var source = card.CardView;
      if (id == clickedId)
      {
        var sorting = _registry.DreamscapeLayout.QuestDeck.Objects.Count;
        updateCards.Add(
          PrototypeQuestCardViewFactory.CloneCardViewWithPositionHidden(
            source,
            new Position { Enum = PositionEnum.QuestDeck },
            sorting
          )
        );
      }
      else if (_currentDraftPickIds.Contains(id))
      {
        updateCards.Add(
          PrototypeQuestCardViewFactory.CloneCardViewWithPosition(
            source,
            new Position { Enum = PositionEnum.Offscreen },
            0
          )
        );
      }
      else
      {
        updateCards.Add(source);
      }
    }

    var update = new UpdateQuestCommand { Quest = new QuestView { Cards = updateCards } };
    yield return _registry.CardService.HandleUpdateQuestCards(update);

    _prototypeCards.UpdateGroupCards("draft", updateCards);
    _prototypeCards.AdvanceGroupWindow("draft", 4);

    var remaining = 0;
    foreach (var cardView in updateCards)
    {
      if (cardView.Position.Position.PositionClass?.SiteDeck != null)
      {
        remaining++;
      }
    }

    if (remaining >= 4)
    {
      yield return RunDraftPickSequence();
    }
    else
    {
      _currentDraftPickIds.Clear();
      _focusMapCamera();
    }
  }

  public IEnumerator RunDraftPickSequence()
  {
    var request = new CreateOrUpdateCardsRequest
    {
      Count = 4,
      Position = new ObjectPosition
      {
        Position = new Position { Enum = PositionEnum.DraftPickDisplay },
        SortingKey = 1,
      },
      Revealed = true,
      OutlineColorHex = _outlineColorProvider(),
      GroupKey = "draft",
      OnClickDebugScenario = "draft-pick",
    };
    var allCards = _prototypeCards.CreateOrUpdateCards(request);
    _currentDraftPickIds = allCards.Take(4).Select(card => card.Id).ToList();

    var customAnimation = new MoveCardsWithCustomAnimationCommand
    {
      Animation = MoveCardsCustomAnimation.ShowInDraftPickLayout,
      Cards = allCards.Take(4).ToList(),
      Destination = new Position { Enum = PositionEnum.DraftPickDisplay },
      PauseDuration = new Milliseconds { MillisecondsValue = 0 },
      StaggerInterval = new Milliseconds { MillisecondsValue = 300 },
    };

    yield return _registry.CardAnimationService.HandleMoveCardsWithCustomAnimation(customAnimation);

    yield return _createOrUpdateCards(request, true);
  }
}
