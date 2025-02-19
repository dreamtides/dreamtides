#nullable enable

using System;
using System.Collections;
using DG.Tweening;
using Dreamcaller.Layout;
using Dreamcaller.Schema;
using Dreamcaller.Services;
using Dreamcaller.Utils;
using TMPro;
using UnityEngine;

namespace Dreamcaller.Components
{
  public class Card : Displayable
  {
    [SerializeField] Transform _cardFront = null!;
    [SerializeField] TextMeshPro _name = null!;
    [SerializeField] TextMeshPro _rulesText = null!;
    [SerializeField] TextMeshPro _typeText = null!;
    [SerializeField] MeshRenderer _cardFrame = null!;
    [SerializeField] MeshRenderer _cardImage = null!;
    [SerializeField] MeshRenderer _cardBack = null!;
    [SerializeField] MeshRenderer _outline = null!;
    [SerializeField] MeshRenderer _battlefieldOutline = null!;
    [SerializeField] MeshRenderer _costBackground = null!;
    [SerializeField] TextMeshPro _costText = null!;
    [SerializeField] MeshRenderer _sparkBackground = null!;
    [SerializeField] TextMeshPro _sparkText = null!;
    [SerializeField] MeshRenderer _battlefieldSparkBackground = null!;
    [SerializeField] TextMeshPro _battlefieldSparkText = null!;
    [SerializeField] ObjectLayout? _containedObjects;
    [SerializeField] ObjectLayout? _stackedObjects;

    bool _isRevealed = false;
    Registry _registry = null!;
    CardView _cardView = null!;
    Quaternion _initialDragRotation;
    float _dragStartScreenZ;
    Vector3 _dragStartPosition;
    Vector3 _dragOffset;
    bool _isDragging = false;
    bool _isDissolving = false;
    public CardView CardView => Errors.CheckNotNull(_cardView);

    public string Id => CardView.ClientId();

    public void Render(Registry registry, CardView view, Sequence? sequence = null)
    {
      var name = view.Revealed?.Name ?? "Hidden Card";
      _registry = registry;
      _cardView = view;
      SortingKey = view.Position.SortingKey;
      gameObject.name = $"{name} [{Id}]";

      if (view.Revealed != null)
      {
        if (_isRevealed)
        {
          RenderCardView();
        }
        else
        {
          Flip(_cardFront, _cardBack, sequence, RenderCardView);
        }
      }
      else
      {
        if (_isRevealed)
        {
          Flip(_cardBack, _cardFront, sequence, RenderCardView);
        }
        else
        {
          RenderCardView();
        }
      }
    }

    public void TurnFaceDown(Sequence? sequence = null) => Flip(_cardFront, _cardBack, sequence);

    public IEnumerator StartDissolve()
    {
      _isDissolving = true;
      _battlefieldSparkBackground.gameObject.SetActive(false);
      _battlefieldOutline.gameObject.SetActive(false);
      var dissolveEffect = ComponentUtils.Get<DissolveEffect>(gameObject);
      yield return dissolveEffect.StartDissolve();
      _isDissolving = false;
    }

    /// <summary>
    /// Creates a clone of the card for large display in the info zoom.
    /// </summary>
    public Card CloneForInfoZoom()
    {
      var clone = Instantiate(gameObject, transform.position, transform.rotation);
      var result = ComponentUtils.Get<Card>(clone);

      if (result._stackedObjects)
      {
        Destroy(result._stackedObjects.gameObject);
        result._stackedObjects = null;
      }

      if (result._containedObjects)
      {
        Destroy(result._containedObjects.gameObject);
        result._containedObjects = null;
      }

      result.gameObject.name = "[IZ]" + gameObject.name;
      result._cardView = CardView;
      result._outline.enabled = false;
      result._registry = _registry;
      result._isRevealed = _isRevealed;
      result.Parent = null;
      result.transform.localPosition = Vector3.zero;
      result.transform.localScale = Vector3.one;
      result.transform.rotation = Quaternion.identity;
      result.GameContext = GameContext.InfoZoom;
      return result;
    }

    void Flip(Component faceUp, Component faceDown, Sequence? sequence, Action? onFlipped = null)
    {
      if (sequence != null)
      {
        const float duration = TweenUtils.FlipAnimationDurationSeconds / 2f;
        sequence
          .Insert(0, faceDown.transform.DOLocalRotate(new Vector3(0, 90, 0), duration))
          .InsertCallback(duration, () =>
          {
            faceUp.transform.localRotation = Quaternion.Euler(0, -90, 0);
            onFlipped?.Invoke();
          })
          .Insert(duration, faceUp.transform.DOLocalRotate(Vector3.zero, duration));
      }
      else
      {
        onFlipped?.Invoke();
      }
    }

    void RenderCardView()
    {
      if (CardView.Revealed != null)
      {
        RenderRevealedCardView(CardView.Revealed);
      }
      else
      {
        RenderHiddenCardView();
      }
    }

    void RenderRevealedCardView(RevealedCardView revealed)
    {
      _isRevealed = true;
      ToggleActiveElements();
      _cardFront.gameObject.SetActive(value: true);
      _cardBack.gameObject.SetActive(value: false);
      _name.text = revealed.Name;
      _rulesText.text = revealed.RulesText;
      _outline.gameObject.SetActive(CanPlay());
      _costText.text = revealed.Cost.ToString();
      _sparkText.text = revealed.Spark.ToString();
      _battlefieldSparkText.text = revealed.Spark.ToString();
      _typeText.text = revealed.CardType;
      _cardImage.material.mainTexture = _registry.AssetService.GetTexture(revealed.Image.Address);
    }

    void RenderHiddenCardView()
    {
      _isRevealed = false;
      _cardFront.gameObject.SetActive(value: false);
      _cardBack.gameObject.SetActive(value: true);
    }

    public override bool CanHandleMouseEvents() => true;

    protected override void OnSetGameContext(GameContext oldContext, GameContext newContext)
    {
      ToggleActiveElements();
    }

    public override void MouseDown()
    {
      if (_registry.CapabilitiesService.CanInfoZoom(GameContext))
      {
        _registry.CardService.DisplayInfoZoom(this);
      }

      if (CanPlay())
      {
        _isDragging = true;
        _registry.SoundService.PlayCardSound();
        GameContext = GameContext.Dragging;
        if (Parent)
        {
          Parent.RemoveIfPresent(this);
        }
        _outline.gameObject.SetActive(false);
        _initialDragRotation = transform.rotation;
        _dragStartScreenZ = Camera.main.WorldToScreenPoint(gameObject.transform.position).z;
        _dragStartPosition = _registry.InputService.WorldMousePosition(_dragStartScreenZ);
        _dragOffset = gameObject.transform.position - _dragStartPosition;
      }
    }

    public override void MouseDrag()
    {
      if (!_isDragging)
      {
        return;
      }

      var mousePosition = _registry.InputService.WorldMousePosition(_dragStartScreenZ);
      var distanceDragged = Vector2.Distance(mousePosition, _dragStartPosition);
      var t = Mathf.Clamp01(distanceDragged / 2);
      transform.position = _dragOffset + mousePosition;
      var rotation = Quaternion.Slerp(_initialDragRotation, Quaternion.Euler(Constants.CameraXAngle, 0, 0), t);
      transform.rotation = rotation;

      if (distanceDragged > 0.25f)
      {
        _registry.CardService.ClearInfoZoom();
      }
    }

    public override void MouseUp()
    {
      _isDragging = false;
      _registry.SoundService.PlayCardSound();
      _registry.CardService.ClearInfoZoom();

      if (CardView.Revealed?.Actions?.OnClick is { } onClick)
      {
        _registry.ActionService.PerformAction(new UserAction
        {
          DebugAction = onClick.DebugAction,
          BattleAction = onClick.BattleAction,
        });
      }

      if (ShouldReturnToPreviousParentOnRelease())
      {
        _registry.LayoutService.AddToParent(this);
        _registry.LayoutService.RunAnimations(() =>
        {
          _outline.gameObject.SetActive(CanPlay());
        });
      }
      else
      {
        var action = new UserAction
        {
          BattleAction = new BattleAction
          {
            PlayCard = CardView.Id
          }
        };

        _registry.ActionService.PerformAction(action);
      }
    }

    void ToggleActiveElements()
    {
      if ((HasGameContext && GameContext.IsBattlefieldContext()) || _isDissolving)
      {
        _cardFrame.gameObject.SetActive(false);
        _name.gameObject.SetActive(false);
        _rulesText.gameObject.SetActive(false);
        _sparkBackground.gameObject.SetActive(false);
        _costBackground.gameObject.SetActive(false);
        _typeText.gameObject.SetActive(false);
        _battlefieldSparkBackground.gameObject.SetActive(CardView.Revealed?.Spark != null);
        _battlefieldOutline.gameObject.SetActive(
          CardView.Revealed?.Status == RevealedCardStatus.CanSelectNegative);
      }
      else
      {
        _cardFrame.gameObject.SetActive(true);
        _name.gameObject.SetActive(true);
        _rulesText.gameObject.SetActive(true);
        _sparkBackground.gameObject.SetActive(CardView.Revealed?.Spark != null);
        _costBackground.gameObject.SetActive(true);
        _typeText.gameObject.SetActive(true);
        _battlefieldSparkBackground.gameObject.SetActive(false);
        _battlefieldOutline.gameObject.SetActive(false);
      }
    }

    bool ShouldReturnToPreviousParentOnRelease()
    {
      if (!CanPlay())
      {
        return true;
      }

      return !_registry.CardService.IsPointerOverPlayCardArea();
    }

    bool CanPlay() => CardView.Revealed?.Actions.CanPlay == true &&
      _registry.CapabilitiesService.CanDragCards();
  }
}