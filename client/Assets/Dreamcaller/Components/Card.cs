#nullable enable

using System;
using System.Collections;
using DG.Tweening;
using Dreamcaller.Layout;
using Dreamcaller.Masonry;
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
    [SerializeField] Transform _battlefieldCardFront = null!;
    [SerializeField] TextMeshPro _name = null!;
    [SerializeField] TextMeshPro _rulesText = null!;
    [SerializeField] TextMeshPro _typeText = null!;
    [SerializeField] Renderer _cardFrame = null!;
    [SerializeField] SpriteRenderer _cardImage = null!;
    [SerializeField] SpriteRenderer _battlefieldCardImage = null!;
    [SerializeField] DissolveEffect _cardImageDissolve = null!;
    [SerializeField] Renderer _cardBack = null!;
    [SerializeField] Renderer _outline = null!;
    [SerializeField] SpriteRenderer _battlefieldOutline = null!;
    [SerializeField] Renderer _costBackground = null!;
    [SerializeField] TextMeshPro _costText = null!;
    [SerializeField] TextMeshPro _producedEnergyText = null!;
    [SerializeField] Renderer _sparkBackground = null!;
    [SerializeField] TextMeshPro _sparkText = null!;
    [SerializeField] Renderer _battlefieldSparkBackground = null!;
    [SerializeField] TextMeshPro _battlefieldSparkText = null!;
    [SerializeField] ObjectLayout? _containedObjects;
    [SerializeField] ObjectLayout? _stackedObjects;
    [SerializeField] Transform _cardTrailPosition = null!;

    bool _isRevealed = false;
    Registry _registry = null!;
    CardView _cardView = null!;
    Quaternion _initialDragRotation;
    float _dragStartScreenZ;
    Vector3 _dragStartPosition;
    Vector3 _dragOffset;
    bool _isDragging = false;
    bool _isDraggingForOrdering = false;
    bool _isDissolved = false;
    public CardView CardView => Errors.CheckNotNull(_cardView);
    GameObject? _cardTrail;

    public string Id => CardView.ClientId();

    public ObjectLayout ContainedObjects => Errors.CheckNotNull(_containedObjects);

    public void Render(Registry registry, CardView view, Sequence? sequence = null)
    {
      var name = view.Revealed?.Name ?? "Hidden Card";
      _registry = registry;
      _cardView = view;
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

    public void TurnFaceDown(Sequence? sequence = null)
    {
      Flip(_cardBack, _cardFront, sequence, () =>
      {
        _cardBack.gameObject.SetActive(true);
        _cardFront.gameObject.SetActive(false);
      });
    }

    public void TurnFaceUp(Sequence? sequence = null)
    {
      Flip(_cardFront, _cardBack, sequence, () =>
      {
        _cardBack.gameObject.SetActive(false);
        _cardFront.gameObject.SetActive(true);
      });
    }

    public IEnumerator StartDissolve(DissolveCardCommand command)
    {
      _isDissolved = true;
      ToggleActiveElements();
      yield return StartCoroutine(_cardImageDissolve.StartDissolve(_registry, command));

      if (command.Reverse)
      {
        _isDissolved = false;
        ToggleActiveElements();
      }
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
      result.GameContext = GameContext.InfoZoom;
      return result;
    }

    void Update()
    {
      _outline.gameObject.SetActive(CanPlay() ||
          CanSelectOrder() ||
          (!GameContext.IsBattlefieldContext() && CardView.Revealed?.OutlineColor != null));
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
      _name.text = revealed.Name;
      _rulesText.text = revealed.RulesText;
      _costBackground.gameObject.SetActive(revealed.Cost != null);
      _costText.text = revealed.Cost?.ToString();
      _producedEnergyText.text = revealed.Produced?.ToString();
      _sparkText.text = revealed.Spark?.ToString();
      _battlefieldSparkText.text = revealed.Spark?.ToString();
      _typeText.text = revealed.CardType;
      _cardImage.sprite = _registry.AssetService.GetSprite(revealed.Image.Address);
      _battlefieldCardImage.sprite = _registry.AssetService.GetSprite(revealed.Image.Address);
      _outline.material.SetInt("_Seed", UnityEngine.Random.Range(0, 9999));
      if (revealed.OutlineColor == null)
      {
        _outline.material.SetColor("_Color", new Color(0f, 1f, 0f));
        _outline.material.SetColor("_HiColor", new Color(0f, 1f, 0f));
        _battlefieldOutline.color = new Color(1f, 1f, 1f);
      }
      else
      {
        _outline.material.SetColor("_Color", MasonRenderer.ToUnityColor(revealed.OutlineColor));
        _outline.material.SetColor("_HiColor", MasonRenderer.ToUnityColor(revealed.OutlineColor));
        _battlefieldOutline.color = MasonRenderer.ToUnityColor(revealed.OutlineColor);
      }

      if (_cardTrail)
      {
        Destroy(_cardTrail);
      }
      if (revealed.Effects.CardTrail != null)
      {
        var trail = _registry.AssetService.GetProjectilePrefab(revealed.Effects.CardTrail);
        _cardTrail = Instantiate(trail.gameObject);
        _cardTrail.transform.SetParent(_cardTrailPosition, worldPositionStays: false);
        _cardTrail.transform.localPosition = Vector3.zero;
        _cardTrail.transform.localRotation = Quaternion.identity;
      }
    }

    void RenderHiddenCardView()
    {
      _isRevealed = false;
      ToggleActiveElements();
    }

    public override bool CanHandleMouseEvents() =>
        GameContext != GameContext.Deck && GameContext != GameContext.DiscardPile;

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

      if (CanPlay() || CanSelectOrder())
      {
        _isDragging = true;
        _isDraggingForOrdering = CanSelectOrder();
        _registry.SoundService.PlayCardSound();
        GameContext = GameContext.Dragging;
        if (Parent)
        {
          Parent.RemoveIfPresent(this);
        }
        _initialDragRotation = transform.rotation;
        _dragStartScreenZ = Camera.main.WorldToScreenPoint(gameObject.transform.position).z;
        _dragStartPosition = _registry.InputService.WorldPointerPosition(_dragStartScreenZ);
        _dragOffset = gameObject.transform.position - _dragStartPosition;
      }
    }

    public override void MouseDrag()
    {
      if (!_isDragging)
      {
        return;
      }

      var mousePosition = _registry.InputService.WorldPointerPosition(_dragStartScreenZ);
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

    public override void MouseUp(bool isSameObject)
    {
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

      if (_isDragging)
      {
        if (ShouldReturnToPreviousParentOnRelease())
        {
          _registry.LayoutService.AddToParent(this);
          _registry.LayoutService.RunAnimations(() =>
          {
            _isDragging = false;
          });
        }
        else if (_isDraggingForOrdering)
        {
          _isDragging = false;
          _isDraggingForOrdering = false;
          _registry.SoundService.PlayCardSound();
          var action = new UserAction
          {
            BattleAction = new()
            {
              BattleActionClass = new()
              {
                SelectCardOrder =
                    _registry.Layout.CardOrderSelector.SelectCardOrderWithinDisplay(transform, CardView.Id),
              }
            }
          };

          _registry.ActionService.PerformAction(action);
        }
        else
        {
          _isDragging = false;
          if (CardView.Revealed?.Actions?.OnPlaySound is { } onPlaySound)
          {
            _registry.SoundService.Play(onPlaySound);
          }
          else
          {
            _registry.SoundService.PlayWhooshSound();
          }
          var action = new UserAction
          {
            BattleAction = new()
            {
              BattleActionClass = new()
              {
                PlayCard = CardView.Id
              }
            }
          };

          _registry.ActionService.PerformAction(action);
        }
      }
    }

    void ToggleActiveElements()
    {
      if (!_isRevealed)
      {
        _cardBack.gameObject.SetActive(true);
        _cardFront.gameObject.SetActive(false);
        _battlefieldCardFront.gameObject.SetActive(false);
      }
      else if (_isDissolved)
      {
        _cardBack.gameObject.SetActive(false);
        _cardFront.gameObject.SetActive(!GameContext.IsBattlefieldContext());
        _battlefieldCardFront.gameObject.SetActive(GameContext.IsBattlefieldContext());
        _cardFrame.gameObject.SetActive(!GameContext.IsBattlefieldContext());
        _battlefieldSparkBackground.gameObject.SetActive(false);
        _battlefieldOutline.gameObject.SetActive(false);
      }
      else if (HasGameContext && GameContext.IsBattlefieldContext())
      {
        _cardBack.gameObject.SetActive(false);
        _cardFront.gameObject.SetActive(false);
        _battlefieldCardFront.gameObject.SetActive(true);
        _battlefieldSparkBackground.gameObject.SetActive(
            GameContext != GameContext.DiscardPile && CardView.Revealed?.Spark != null);
      }
      else
      {
        _cardBack.gameObject.SetActive(false);
        _cardFront.gameObject.SetActive(true);
        _battlefieldCardFront.gameObject.SetActive(false);
        _sparkBackground.gameObject.SetActive(CardView.Revealed?.Spark != null);
      }
    }

    bool ShouldReturnToPreviousParentOnRelease()
    {
      if (CardView.Revealed?.Actions.CanPlay != true && CardView.Revealed?.Actions.CanSelectOrder != true)
      {
        return true;
      }

      return !_registry.CardService.IsPointerOverPlayCardArea();
    }

    bool CanPlay() => CardView.Revealed?.Actions.CanPlay == true &&
      _registry.CapabilitiesService.CanPlayCards() &&
      GameContext == GameContext.Hand;

    bool CanSelectOrder() => CardView.Revealed?.Actions.CanSelectOrder == true;
  }
}