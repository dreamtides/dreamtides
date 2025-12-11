#nullable enable

using System.Collections.Generic;
using System.Runtime.CompilerServices;
using Dreamtides.Buttons;
using Dreamtides.Schema;
using TMPro;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  public sealed class StartBattleObjectLayout : StandardObjectLayout
  {
    [SerializeField]
    internal float _horizontalSpacing = 3f;

    [SerializeField]
    internal float _cardInwardOffsetPortrait = 0f;

    [SerializeField]
    internal float _cardInwardOffsetLandscape = 0f;

    [SerializeField]
    internal float _cardWidth = 2.5f;

    [SerializeField]
    internal float _cardHeight = 3.5f;

    [SerializeField]
    internal float _cardScalePortrait = 1f;

    [SerializeField]
    internal float _cardScaleLandscape = 1f;

    [SerializeField]
    internal TextMeshPro? _vsText;

    [SerializeField]
    internal float _vsTextFontSizePortrait = 2f;

    [SerializeField]
    internal float _vsTextFontSizeLandscape = 3f;

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
    internal float _dreamsignScale = 0.25f;

    [SerializeField]
    internal float _dreamsignHorizontalSpacing = 0.8f;

    [SerializeField]
    internal float _dreamsignVerticalSpacing = 1.0f;

    [SerializeField]
    internal float _dreamsignColumnSpacing = 0.8f;

    [SerializeField]
    internal float _dreamsignOutwardOffset = 1.5f;

    [SerializeField]
    internal float _dreamsignPortraitVerticalOffset = -2.5f;

    DisplayableButton? _buttonInstance;

    float CardInwardOffset =>
      IsLandscape() ? _cardInwardOffsetLandscape : _cardInwardOffsetPortrait;
    float CardScale => IsLandscape() ? _cardScaleLandscape : _cardScalePortrait;
    float VsTextFontSize => IsLandscape() ? _vsTextFontSizeLandscape : _vsTextFontSizePortrait;
    float ButtonVerticalOffset =>
      IsLandscape() ? _buttonVerticalOffsetLandscape : _buttonVerticalOffsetPortrait;
    float ButtonScale => IsLandscape() ? _buttonScaleLandscape : _buttonScalePortrait;

    readonly List<Displayable> _userDreamsigns = new();
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
        return CalculateIdentityCardPosition(index, count);
      }

      return displayType.Value switch
      {
        StartBattleDisplayType.UserIdentityCard => CalculateIdentityCardPosition(
          index: 0,
          identityCardCount: 2
        ),
        StartBattleDisplayType.EnemyIdentityCard => CalculateIdentityCardPosition(
          index: 1,
          identityCardCount: 2
        ),
        StartBattleDisplayType.UserDreamsigns => CalculateUserDreamsignPosition(displayable),
        StartBattleDisplayType.EnemyDreamsigns => CalculateEnemyDreamsignPosition(displayable),
        _ => CalculateIdentityCardPosition(index, count),
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
        StartBattleDisplayType.UserDreamsigns => _dreamsignScale,
        StartBattleDisplayType.EnemyDreamsigns => _dreamsignScale,
        _ => CardScale,
      };
    }

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      if (count <= 0)
      {
        return transform.position;
      }

      if (count == 1)
      {
        return transform.position;
      }

      var displayable = index < Objects.Count ? Objects[index] : null;
      if (displayable != null)
      {
        return CalculateObjectPositionForDisplayable(displayable, index, count);
      }

      return CalculateIdentityCardPosition(index, identityCardCount: 2);
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
      PositionVsText();
      PositionButton();
      ShowVsText();
      ShowButton();
    }

    protected override void OnBecameEmpty()
    {
      base.OnBecameEmpty();
      HideVsText();
      HideButton();
    }

    protected override void OnUpdateObjectLayout()
    {
      if (DebugUpdateContinuously)
      {
        PositionVsText();
        UpdateVsTextFontSize();
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

    void PositionVsText()
    {
      if (!_vsText)
      {
        return;
      }

      _vsText.transform.SetPositionAndRotation(transform.position, transform.rotation);
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

    void ShowVsText()
    {
      if (_vsText)
      {
        _vsText.fontSize = VsTextFontSize;
        _vsText.gameObject.SetActive(true);
      }
    }

    void HideVsText()
    {
      if (_vsText)
      {
        _vsText.gameObject.SetActive(false);
      }
    }

    void UpdateVsTextFontSize()
    {
      if (_vsText)
      {
        _vsText.fontSize = VsTextFontSize;
      }
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
      _userDreamsigns.Clear();
      _enemyDreamsigns.Clear();

      foreach (var obj in Objects)
      {
        var displayType = GetDisplayType(obj);
        if (displayType == StartBattleDisplayType.UserDreamsigns)
        {
          _userDreamsigns.Add(obj);
        }
        else if (displayType == StartBattleDisplayType.EnemyDreamsigns)
        {
          _enemyDreamsigns.Add(obj);
        }
      }
    }

    StartBattleDisplayType? GetDisplayType(Displayable displayable)
    {
      return displayable.ObjectPosition?.Position.PositionClass?.StartBattleDisplay;
    }

    Vector3 CalculateIdentityCardPosition(int index, int identityCardCount)
    {
      if (identityCardCount <= 1)
      {
        return transform.position;
      }

      var isLeftCard = index == 0;
      var baseOffset = isLeftCard ? -_horizontalSpacing / 2f : _horizontalSpacing / 2f;
      var inwardAdjustment = isLeftCard ? CardInwardOffset : -CardInwardOffset;
      return transform.position + transform.right * (baseOffset + inwardAdjustment);
    }

    Vector3 CalculateUserDreamsignPosition(Displayable displayable)
    {
      var indexInList = _userDreamsigns.IndexOf(displayable);
      if (indexInList < 0)
      {
        indexInList = 0;
      }

      return CalculateDreamsignPosition(indexInList, _userDreamsigns.Count, isUserSide: true);
    }

    Vector3 CalculateEnemyDreamsignPosition(Displayable displayable)
    {
      var indexInList = _enemyDreamsigns.IndexOf(displayable);
      if (indexInList < 0)
      {
        indexInList = 0;
      }

      return CalculateDreamsignPosition(indexInList, _enemyDreamsigns.Count, isUserSide: false);
    }

    Vector3 CalculateDreamsignPosition(int index, int count, bool isUserSide)
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
        return CalculateDreamsignPositionLandscape(column, rowInColumn, columnCount, isUserSide);
      }
      else
      {
        return CalculateDreamsignPositionPortrait(column, rowInColumn, columnCount, isUserSide);
      }
    }

    Vector3 CalculateDreamsignPositionLandscape(
      int column,
      int rowInColumn,
      int columnCount,
      bool isUserSide
    )
    {
      var identityCardX = isUserSide
        ? -_horizontalSpacing / 2f + CardInwardOffset
        : _horizontalSpacing / 2f - CardInwardOffset;

      var cardEdgeOffset = (_cardWidth * CardScale) / 2f + _dreamsignOutwardOffset;

      float baseX;
      if (isUserSide)
      {
        baseX = identityCardX - cardEdgeOffset;
        baseX -= column * _dreamsignColumnSpacing;
      }
      else
      {
        baseX = identityCardX + cardEdgeOffset;
        baseX += column * _dreamsignColumnSpacing;
      }

      var totalHeight = (columnCount - 1) * _dreamsignVerticalSpacing;
      var cardBottomY = -(_cardHeight * CardScale) / 2f;
      var topY = cardBottomY + totalHeight;
      var y = topY - rowInColumn * _dreamsignVerticalSpacing;

      return transform.position + transform.right * baseX + transform.up * y;
    }

    Vector3 CalculateDreamsignPositionPortrait(
      int column,
      int rowInColumn,
      int columnCount,
      bool isUserSide
    )
    {
      var identityCardX = isUserSide
        ? -_horizontalSpacing / 2f + CardInwardOffset
        : _horizontalSpacing / 2f - CardInwardOffset;

      var columnOffset = (column == 0 ? -1 : 1) * _dreamsignColumnSpacing / 2f;
      var x = identityCardX + columnOffset;

      var baseY = _dreamsignPortraitVerticalOffset;
      var topY = baseY;
      var y = topY - rowInColumn * _dreamsignVerticalSpacing;

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

      var leftCardCenter = -_horizontalSpacing / 2f + CardInwardOffset;
      var rightCardCenter = _horizontalSpacing / 2f - CardInwardOffset;

      Gizmos.DrawSphere(
        center + right * (leftCardCenter - halfCardWidth) + upAxis * halfCardHeight,
        0.1f
      );
      Gizmos.DrawSphere(
        center + right * (leftCardCenter + halfCardWidth) + upAxis * halfCardHeight,
        0.1f
      );
      Gizmos.DrawSphere(
        center + right * (leftCardCenter - halfCardWidth) - upAxis * halfCardHeight,
        0.1f
      );
      Gizmos.DrawSphere(
        center + right * (leftCardCenter + halfCardWidth) - upAxis * halfCardHeight,
        0.1f
      );

      Gizmos.DrawSphere(
        center + right * (rightCardCenter - halfCardWidth) + upAxis * halfCardHeight,
        0.1f
      );
      Gizmos.DrawSphere(
        center + right * (rightCardCenter + halfCardWidth) + upAxis * halfCardHeight,
        0.1f
      );
      Gizmos.DrawSphere(
        center + right * (rightCardCenter - halfCardWidth) - upAxis * halfCardHeight,
        0.1f
      );
      Gizmos.DrawSphere(
        center + right * (rightCardCenter + halfCardWidth) - upAxis * halfCardHeight,
        0.1f
      );

      Gizmos.color = Color.yellow;
      var buttonPos = center + upAxis * ButtonVerticalOffset;
      Gizmos.DrawSphere(buttonPos, 0.1f);
    }
  }
}
