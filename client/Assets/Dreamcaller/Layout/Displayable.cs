#nullable enable

using Dreamcaller.Utils;
using UnityEngine;
using UnityEngine.Rendering;

namespace Dreamcaller.Layout
{
  public abstract class Displayable : MonoBehaviour
  {
    [SerializeField] Transform? _projectileSourcePosition;
    [SerializeField] Transform? _displayEffectPosition;
    [SerializeField] SortingGroup? _sortingGroup;
    [SerializeField] GameContext _internalGameContext;
    ObjectLayout? _parent;

    /// <summary>Order items within a sorting layer.</summary>
    public int SortingKey
    {
      get
      {
        if (_sortingGroup)
        {
          return _sortingGroup.sortingOrder;
        }
        else
        {
          return 0;
        }
      }
      set
      {
        if (_sortingGroup)
        {
          _sortingGroup.sortingOrder = value;
        }
      }
    }

    public ObjectLayout? Parent
    {
      get => _parent;
      set => _parent = value;
    }

    public virtual float DefaultScale => 1.0f;

    /// <summary>
    /// The position that a projectile will be fired from this object.
    /// </summary>
    public Transform? ProjectileSourcePosition => _projectileSourcePosition;

    /// <summary>
    /// Position at which to display particle effects on this object.
    /// </summary>
    public Transform? DisplayEffectPosition => _displayEffectPosition;

    /// <summary>
    /// SortingGroup for this object.
    /// </summary>
    public SortingGroup? SortingGroup => _sortingGroup;

    protected void Start()
    {
      OnStart();
    }

    /// <summary>
    /// Invoked when the displayable is created.
    /// </summary>
    protected virtual void OnStart()
    {
    }

    /// <summary>
    /// Should return true if this game object can currently handle a MouseDown or MouseHoverStart event.
    /// </summary>
    public virtual bool CanHandleMouseEvents() => false;

    /// <summary>
    /// Invoked on mouse down. Will only be invoked if <see cref="CanHandleMouseEvents"/>
    /// returns true and this is the topmost object hit by the on click raycast.
    /// </summary>
    public virtual void MouseDown()
    {
    }

    /// <summary>
    /// Sent every frame while the mouse button is held down to objects which received <see cref="MouseDown"/>.
    /// </summary>
    public virtual void MouseDrag()
    {
    }

    /// <summary>
    /// Sent on *any* mouse up event, anywhere on screen, to objects which
    /// received a <see cref="MouseDown"/>. If "isSameObject" is true, it means
    /// the pointer was released over the same Displayable which received
    /// MouseDown.
    /// </summary>
    public virtual void MouseUp(bool isSameObject)
    {
    }

    /// <summary>
    /// Invoked on mouse hover start. Will only be invoked if <see cref="CanHandleMouseEvents"/>
    /// returns true and this is the topmost object hit by the on click raycast.
    /// </summary>
    public virtual void MouseHoverStart()
    {
    }

    public virtual void MouseHover()
    {
    }

    public virtual void MouseHoverEnd()
    {
    }

    /// <summary>Called when the parent container is repositioned.</summary>
    public virtual void OnParentRepositioned()
    {
    }

    public GameContext GameContext
    {
      get => Errors.CheckNotDefault(_internalGameContext);
      set
      {
        Errors.CheckNotDefault(value);

        if (_internalGameContext != value)
        {
          var oldContext = _internalGameContext;
          _internalGameContext = value;
          if (_sortingGroup)
          {
            _sortingGroup.sortingLayerID = value.SortingLayerId();
          }
          OnSetGameContext(oldContext, value);
        }
      }
    }

    public bool HasGameContext => _internalGameContext != GameContext.Unspecified;

    protected virtual void OnSetGameContext(GameContext oldContext, GameContext newContext) { }
  }
}