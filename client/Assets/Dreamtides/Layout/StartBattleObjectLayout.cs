#nullable enable

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

    DisplayableButton? _buttonInstance;

    float CardInwardOffset =>
      IsLandscape() ? _cardInwardOffsetLandscape : _cardInwardOffsetPortrait;
    float CardScale => IsLandscape() ? _cardScaleLandscape : _cardScalePortrait;
    float VsTextFontSize => IsLandscape() ? _vsTextFontSizeLandscape : _vsTextFontSizePortrait;
    float ButtonVerticalOffset =>
      IsLandscape() ? _buttonVerticalOffsetLandscape : _buttonVerticalOffsetPortrait;
    float ButtonScale => IsLandscape() ? _buttonScaleLandscape : _buttonScalePortrait;

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

      var isLeftCard = index == 0;
      var baseOffset = isLeftCard ? -_horizontalSpacing / 2f : _horizontalSpacing / 2f;
      var inwardAdjustment = isLeftCard ? CardInwardOffset : -CardInwardOffset;
      return transform.position + transform.right * (baseOffset + inwardAdjustment);
    }

    public override Vector3? CalculateObjectRotation(int index, int count) =>
      transform.rotation.eulerAngles;

    public override float? CalculateObjectScale(int index, int count) => CardScale;

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
