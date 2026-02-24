#nullable enable

using System.Collections.Generic;
using System.Runtime.CompilerServices;
using Dreamtides.Buttons;
using Dreamtides.Schema;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  public sealed class StartBattleObjectLayout : StandardObjectLayout
  {
    [SerializeField]
    internal float _cardWidth = 2.5f;

    [SerializeField]
    internal float _cardHeight = 3.5f;

    [SerializeField]
    internal float _cardScalePortrait = 1f;

    [SerializeField]
    internal float _cardScaleLandscape = 1f;

    [SerializeField]
    internal DisplayableButton? _buttonPrefab;

    [SerializeField]
    internal float _buttonVerticalOffsetPortrait = -0.6f;

    [SerializeField]
    internal float _buttonVerticalOffsetLandscape = -0.6f;

    [SerializeField]
    internal float _buttonScalePortrait = 0.15f;

    [SerializeField]
    internal float _buttonScaleLandscape = 0.15f;

    [SerializeField]
    internal float _dreamsignScalePortrait = 0.25f;

    [SerializeField]
    internal float _dreamsignScaleLandscape = 0.25f;

    [SerializeField]
    internal float _dreamsignHorizontalSpacingPortrait = 0.8f;

    [SerializeField]
    internal float _dreamsignHorizontalSpacingLandscape = 0.8f;

    [SerializeField]
    internal float _dreamsignVerticalSpacingPortrait = 1.0f;

    [SerializeField]
    internal float _dreamsignVerticalSpacingLandscape = 1.0f;

    [SerializeField]
    internal float _dreamsignColumnSpacingPortrait = 0.8f;

    [SerializeField]
    internal float _dreamsignColumnSpacingLandscape = 0.8f;

    [SerializeField]
    internal float _dreamsignOffsetFromIdentityPortrait = 2.5f;

    [SerializeField]
    internal float _dreamsignOffsetFromIdentityLandscape = 2.5f;

    [SerializeField]
    internal float _dreamsignVerticalOffsetPortrait = 0f;

    [SerializeField]
    internal float _dreamsignVerticalOffsetLandscape = 0f;

    internal DisplayableButton? _buttonInstance;

    float CardScale => IsLandscape() ? _cardScaleLandscape : _cardScalePortrait;
    float ButtonVerticalOffset =>
      IsLandscape() ? _buttonVerticalOffsetLandscape : _buttonVerticalOffsetPortrait;
    float ButtonScale => IsLandscape() ? _buttonScaleLandscape : _buttonScalePortrait;
    float DreamsignScale => IsLandscape() ? _dreamsignScaleLandscape : _dreamsignScalePortrait;
    float DreamsignVerticalSpacing =>
      IsLandscape() ? _dreamsignVerticalSpacingLandscape : _dreamsignVerticalSpacingPortrait;
    float DreamsignColumnSpacing =>
      IsLandscape() ? _dreamsignColumnSpacingLandscape : _dreamsignColumnSpacingPortrait;
    float DreamsignOffsetFromIdentity =>
      IsLandscape() ? _dreamsignOffsetFromIdentityLandscape : _dreamsignOffsetFromIdentityPortrait;
    float DreamsignVerticalOffset =>
      IsLandscape() ? _dreamsignVerticalOffsetLandscape : _dreamsignVerticalOffsetPortrait;

    readonly List<Displayable> _enemyDreamsigns = new();

    readonly ButtonView _defaultButtonView = new()
    {
      Label = "Start Battle",
      Action = new OnClickClass
      {
        DebugAction = new DebugAction
        {
          DebugActionClass = new DebugActionClass { ApplyTestScenarioAction = "StartBattle" },
        },
      },
    };
    ButtonView? _buttonView;

    public void SetButtonView(ButtonView? view)
    {
      _buttonView = view;
      UpdateButton();
    }

    public void ShowButton()
    {
      EnsureButtonInstance();
      if (_buttonInstance)
      {
        _buttonInstance.gameObject.SetActive(true);
        UpdateButton();
      }
    }

    public void HideButton()
    {
      if (_buttonInstance)
      {
        _buttonInstance.gameObject.SetActive(false);
      }
    }

    public Vector3 CalculateObjectPositionForDisplayable(
      Displayable displayable,
      int index,
      int count
    )
    {
      var displayType = GetDisplayType(displayable);
      if (displayType == null)
      {
        return transform.position;
      }

      return displayType.Value switch
      {
        StartBattleDisplayType.EnemyIdentityCard => transform.position,
        StartBattleDisplayType.EnemyDreamsigns => CalculateEnemyDreamsignPosition(displayable),
        _ => transform.position,
      };
    }

    public float CalculateObjectScaleForDisplayable(Displayable displayable)
    {
      var displayType = GetDisplayType(displayable);
      if (displayType == null)
      {
        return CardScale;
      }

      return displayType.Value switch
      {
        StartBattleDisplayType.UserDreamsigns => DreamsignScale,
        StartBattleDisplayType.EnemyDreamsigns => DreamsignScale,
        _ => CardScale,
      };
    }

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      if (count <= 0)
      {
        return transform.position;
      }

      var displayable = index < Objects.Count ? Objects[index] : null;
      if (displayable != null)
      {
        return CalculateObjectPositionForDisplayable(displayable, index, count);
      }

      return transform.position;
    }

    public override Vector3? CalculateObjectRotation(int index, int count) =>
      transform.rotation.eulerAngles;

    public override float? CalculateObjectScale(int index, int count)
    {
      var displayable = index < Objects.Count ? Objects[index] : null;
      if (displayable != null)
      {
        return CalculateObjectScaleForDisplayable(displayable);
      }

      return CardScale;
    }

    protected override void OnBecameNonEmpty()
    {
      base.OnBecameNonEmpty();
      EnsureButtonInstance();
      PositionButton();
      ShowButton();
    }

    protected override void OnBecameEmpty()
    {
      base.OnBecameEmpty();
      HideButton();
    }

    protected override void OnUpdateObjectLayout()
    {
      if (DebugUpdateContinuously)
      {
        PositionButton();
        UpdateButtonScale();
      }
    }

    void EnsureButtonInstance()
    {
      if (_buttonInstance || !_buttonPrefab)
      {
        return;
      }

      _buttonInstance = Instantiate(_buttonPrefab, transform);
      _buttonInstance.Initialize(this);
      _buttonInstance.gameObject.SetActive(false);
      _buttonInstance.SetDefaultScale(Vector3.one * ButtonScale);
    }

    void PositionButton()
    {
      if (!_buttonInstance)
      {
        return;
      }

      var buttonPosition = transform.position + transform.up * ButtonVerticalOffset;
      _buttonInstance.transform.SetPositionAndRotation(buttonPosition, transform.rotation);
    }

    void UpdateButton()
    {
      if (!_buttonInstance)
      {
        return;
      }

      _buttonInstance.SetView(_buttonView ?? _defaultButtonView);
    }

    void UpdateButtonScale()
    {
      if (!_buttonInstance)
      {
        return;
      }

      _buttonInstance.SetDefaultScale(Vector3.one * ButtonScale);
    }

    protected override void OnBeforeApplyLayout()
    {
      CategorizeObjects();
    }

    void CategorizeObjects()
    {
      _enemyDreamsigns.Clear();

      foreach (var obj in Objects)
      {
        var displayType = GetDisplayType(obj);
        if (displayType == StartBattleDisplayType.EnemyDreamsigns)
        {
          _enemyDreamsigns.Add(obj);
        }
      }
    }

    StartBattleDisplayType? GetDisplayType(Displayable displayable)
    {
      return displayable.ObjectPosition?.Position.PositionClass?.StartBattleDisplay;
    }

    Vector3 CalculateEnemyDreamsignPosition(Displayable displayable)
    {
      var indexInList = _enemyDreamsigns.IndexOf(displayable);
      if (indexInList < 0)
      {
        indexInList = 0;
      }

      return CalculateDreamsignPosition(indexInList, _enemyDreamsigns.Count);
    }

    Vector3 CalculateDreamsignPosition(int index, int count)
    {
      if (count <= 0)
      {
        return transform.position;
      }

      var column1Count = (count + 1) / 2;
      var column2Count = count / 2;

      int column;
      int rowInColumn;
      int columnCount;

      if (index < column1Count)
      {
        column = 0;
        rowInColumn = index;
        columnCount = column1Count;
      }
      else
      {
        column = 1;
        rowInColumn = index - column1Count;
        columnCount = column2Count;
      }

      if (IsLandscape())
      {
        return CalculateDreamsignPositionLandscape(column, rowInColumn, columnCount);
      }
      else
      {
        return CalculateDreamsignPositionPortrait(column, rowInColumn, columnCount);
      }
    }

    Vector3 CalculateDreamsignPositionLandscape(int column, int rowInColumn, int columnCount)
    {
      var cardEdgeOffset = (_cardWidth * CardScale) / 2f;
      var baseX = cardEdgeOffset + DreamsignOffsetFromIdentity;
      baseX += column * DreamsignColumnSpacing;

      var totalHeight = (columnCount - 1) * DreamsignVerticalSpacing;
      var cardBottomY = -(_cardHeight * CardScale) / 2f + DreamsignVerticalOffset;
      var topY = cardBottomY + totalHeight;
      var y = topY - rowInColumn * DreamsignVerticalSpacing;

      return transform.position + transform.right * baseX + transform.up * y;
    }

    Vector3 CalculateDreamsignPositionPortrait(int column, int rowInColumn, int columnCount)
    {
      var cardEdgeOffset = (_cardWidth * CardScale) / 2f;
      var baseX = cardEdgeOffset + DreamsignOffsetFromIdentity;
      var columnOffset = (column == 0 ? -1 : 1) * DreamsignColumnSpacing / 2f;
      var x = baseX + columnOffset;

      var baseY = DreamsignVerticalOffset;
      var topY = baseY;
      var y = topY - rowInColumn * DreamsignVerticalSpacing;

      return transform.position + transform.right * x + transform.up * y;
    }

    void OnDrawGizmosSelected()
    {
      Gizmos.color = Color.cyan;
      var center = transform.position;
      var halfCardWidth = _cardWidth / 2f;
      var halfCardHeight = _cardHeight / 2f;
      var right = transform.right;
      var upAxis = transform.up;

      Gizmos.DrawSphere(center, 0.1f);

      Gizmos.DrawSphere(center + right * (-halfCardWidth) + upAxis * halfCardHeight, 0.1f);
      Gizmos.DrawSphere(center + right * (halfCardWidth) + upAxis * halfCardHeight, 0.1f);
      Gizmos.DrawSphere(center + right * (-halfCardWidth) - upAxis * halfCardHeight, 0.1f);
      Gizmos.DrawSphere(center + right * (halfCardWidth) - upAxis * halfCardHeight, 0.1f);

      Gizmos.color = Color.yellow;
      var buttonPos = center + upAxis * ButtonVerticalOffset;
      Gizmos.DrawSphere(buttonPos, 0.1f);
    }
  }
}
