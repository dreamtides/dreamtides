#nullable enable

using Dreamcaller.Utils;
using UnityEngine;
using UnityEngine.Rendering;

namespace Dreamcaller.Layout
{
  public abstract class Displayable : MonoBehaviour
  {
    GameContext _gameContext;
    ObjectLayout? _parent;
    long _sortingKey;

    /// <summary>Order items within a sorting layer.</summary>
    public long SortingKey
    {
      get => _sortingKey;
      set
      {
        _sortingKey = value;
      }
    }

    public ObjectLayout? Parent
    {
      get => _parent;
      set => _parent = value;
    }

    public virtual float DefaultScale => 1.0f;

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
    /// Sent on *any* mouse up event, anywhere on screen, to objects which received a <see cref="MouseDown"/>.
    /// </summary>
    public virtual void MouseUp()
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
      get => Errors.CheckNotDefault(_gameContext);
      set
      {
        Errors.CheckNotDefault(value);

        if (_gameContext != value)
        {
          var oldContext = _gameContext;
          _gameContext = value;
          OnSetGameContext(oldContext, value);
        }
      }
    }

    protected virtual void OnSetGameContext(GameContext oldContext, GameContext newContext) { }
  }
}