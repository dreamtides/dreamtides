#nullable enable

using System.Collections;
using Dreamtides.Buttons;
using Dreamtides.Layout;
using Dreamtides.Masonry;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.TestFakes;
using Dreamtides.Tests.TestUtils;
using NUnit.Framework;
using TMPro;
using UnityEngine;
using UnityEngine.TestTools;
using UnityEngine.UIElements;

namespace Dreamtides.Tests.Abu
{
  /// <summary>
  /// Validates that each input simulation pathway works in EditMode tests.
  /// These tests exercise the three distinct UI input systems used by the game:
  /// UI Toolkit (Masonry), 3D Displayables (via IInputProvider), and UGUI
  /// CanvasButtons.
  /// </summary>
  [TestFixture]
  public class InputSimulationTests : DreamtidesUnitTest
  {
    /// <summary>
    /// Test Displayable subclass that records whether hover methods were called.
    /// </summary>
    sealed class HoverRecordingDisplayable : Displayable
    {
      public bool HoverStartCalled { get; private set; }
      public bool HoverCalled { get; private set; }
      public bool HoverEndCalled { get; private set; }

      public override bool CanHandleMouseEvents() => true;

      public override void MouseHoverStart()
      {
        HoverStartCalled = true;
      }

      public override void MouseHover()
      {
        HoverCalled = true;
      }

      public override void MouseHoverEnd()
      {
        HoverEndCalled = true;
      }
    }

    /// <summary>
    /// Fake IInputProvider that returns configurable values for pointer state
    /// and object-at-pointer queries.
    /// </summary>
    sealed class FakeInputProvider : IInputProvider
    {
      public bool Pressed { get; set; }
      public Vector2 Position { get; set; } = Vector2.zero;
      public Displayable? MouseDownTarget { get; set; }
      public Displayable? MouseUpTarget { get; set; }
      public Displayable? HoverTarget { get; set; }

      public bool IsPointerPressed() => Pressed;

      public Vector2 PointerPosition() => Position;

      public Displayable? ObjectAtPointerPosition(MouseEventType eventType)
      {
        return eventType switch
        {
          MouseEventType.MouseDown => MouseDownTarget,
          MouseEventType.MouseUp => MouseUpTarget,
          MouseEventType.MouseHover => HoverTarget,
          _ => null,
        };
      }
    }

    FakeActionService SetUpFakeActionService()
    {
      var fakeAction = Registry.gameObject.AddComponent<FakeActionService>();
      Registry._actionService = fakeAction;
      return fakeAction;
    }

    void SetUpFakeSoundService()
    {
      var fakeSound = Registry.gameObject.AddComponent<FakeSoundService>();
      Registry._soundService = fakeSound;
    }

    // -- Test 1: UI Toolkit Click Simulation --

    /// <summary>
    /// Verifies that UI Toolkit click simulation works by calling
    /// Callbacks.OnClick() directly. This is a public method on the Callbacks
    /// class (Elements.cs:112) and works without requiring panel attachment.
    ///
    /// Approach used: Direct Callbacks.OnClick() invocation (fallback approach).
    /// The primary approach (element.SendEvent) requires a live panel which is
    /// not reliably available in EditMode tests. Calling OnClick directly is
    /// reliable and exercises the same code path that the UI Toolkit event
    /// system uses.
    /// </summary>
    [UnityTest]
    public IEnumerator UiToolkitClickSimulation_DirectCallbackInvocation()
    {
      yield return Initialize();

      var element = new NodeVisualElement();
      var callbacks = element.Callbacks.Value;
      var clickFired = false;

      callbacks.SetCallback(element, Callbacks.Event.Click, () => { clickFired = true; });

      // Simulate a click by calling OnClick directly with a pooled ClickEvent.
      using var clickEvent = ClickEvent.GetPooled();
      callbacks.OnClick(clickEvent);

      Assert.IsTrue(clickFired, "Click callback should have been invoked via direct OnClick call");
    }

    // -- Test 2: UI Toolkit Hover Simulation --

    /// <summary>
    /// Verifies that UI Toolkit hover enter and leave simulation works by
    /// calling Callbacks.OnMouseEnter() and OnMouseLeave() directly on the
    /// same element. The OnMouseEnter/OnMouseLeave methods were changed from
    /// private to internal on the Callbacks class, and
    /// [assembly: InternalsVisibleTo("Dreamtides.Tests")] was added to
    /// Elements.cs to enable direct invocation from tests.
    ///
    /// Approach used: Direct Callbacks.OnMouseEnter()/OnMouseLeave()
    /// invocation. This is the most reliable approach for EditMode tests
    /// since element.SendEvent() may not work reliably without a live panel
    /// in batch mode. Testing both enter and leave on the same element
    /// exercises a realistic hover sequence.
    /// </summary>
    [UnityTest]
    public IEnumerator UiToolkitHoverEnterAndLeaveSimulation()
    {
      yield return Initialize();

      var element = new NodeVisualElement();
      var callbacks = element.Callbacks.Value;
      var enterFired = false;
      var leaveFired = false;

      callbacks.SetCallback(
        element,
        Callbacks.Event.MouseEnter,
        () => { enterFired = true; }
      );
      callbacks.SetCallback(
        element,
        Callbacks.Event.MouseLeave,
        () => { leaveFired = true; }
      );

      // Simulate hover enter
      using var mouseEnterEvent = MouseEnterEvent.GetPooled();
      callbacks.OnMouseEnter(mouseEnterEvent);

      Assert.IsTrue(
        enterFired,
        "MouseEnter callback should have been invoked via direct OnMouseEnter call"
      );
      Assert.IsFalse(
        leaveFired,
        "MouseLeave callback should not have fired yet"
      );

      // Simulate hover leave
      using var mouseLeaveEvent = MouseLeaveEvent.GetPooled();
      callbacks.OnMouseLeave(mouseLeaveEvent);

      Assert.IsTrue(
        leaveFired,
        "MouseLeave callback should have been invoked via direct OnMouseLeave call"
      );
    }

    // -- Test 3: Displayable Click Simulation (via IInputProvider) --

    /// <summary>
    /// Verifies that the IInputProvider injection pathway works for simulating
    /// 3D Displayable clicks. This uses the two-frame click sequence:
    /// Frame 1: IsPointerPressed() => true, ObjectAtPointerPosition(MouseDown) => target
    /// Frame 2: IsPointerPressed() => false, ObjectAtPointerPosition(MouseUp) => target
    ///
    /// The InputService.OnUpdate() method drives the click detection. We call
    /// InputService.Update() manually since MonoBehaviour.Update() may not be
    /// called reliably in EditMode tests.
    /// </summary>
    [UnityTest]
    public IEnumerator DisplayableClickSimulation_ViaIInputProvider()
    {
      yield return Initialize();

      var fakeActionService = SetUpFakeActionService();
      SetUpFakeSoundService();

      // Create a DisplayableButton with required serialized fields
      var button = CreateSceneObject<DisplayableButton>(b =>
      {
        b._background = b.gameObject.AddComponent<SpriteRenderer>();
        var textGo = new GameObject("ButtonText");
        textGo.transform.SetParent(b.transform);
        b._text = textGo.AddComponent<TextMeshPro>();
        var colliderGo = new GameObject("ButtonCollider");
        colliderGo.transform.SetParent(b.transform);
        b._collider = colliderGo.AddComponent<BoxCollider>();
        b._noOutlineMaterial = new Material(Shader.Find("Sprites/Default"));
      });

      // Set a view with a non-null action so MouseDown/MouseUp can proceed
      var testAction = new OnClickUnion { Enum = GameActionEnum.NoOp };
      button.SetView(new ButtonView { Label = "Test", Action = testAction });

      // Set up fake input provider
      var fakeInput = new FakeInputProvider();
      Registry.InputService.InputProvider = fakeInput;

      // Ensure DocumentService won't block clicks
      Registry.DocumentService.HasOpenPanels = false;

      // In EditMode, accessing renderer.material logs an error about material
      // leaking. This is expected behavior for DisplayableButton.MouseDown()
      // which reads _background.material to save the material before press.
      LogAssert.Expect(
        UnityEngine.LogType.Error,
        "Instantiating material due to calling renderer.material during edit mode. "
          + "This will leak materials into the scene. "
          + "You most likely want to use renderer.sharedMaterial instead."
      );

      // Frame 1: Pointer pressed, target is the button
      fakeInput.Pressed = true;
      fakeInput.MouseDownTarget = button;
      fakeInput.MouseUpTarget = button;

      // Manually drive InputService update (MonoBehaviour.Update may not fire
      // in EditMode)
      Registry.InputService.Update();

      // Frame 2: Pointer released, same target (isSameObject = true)
      fakeInput.Pressed = false;
      Registry.InputService.Update();

      Assert.That(
        fakeActionService.PerformedActions.Count,
        Is.GreaterThan(0),
        "DisplayableButton click via IInputProvider should record an action in FakeActionService"
      );
    }

    // -- Test 4: CanvasButton Click Simulation --

    /// <summary>
    /// Verifies that CanvasButton.OnClick() can be called directly to simulate
    /// a click. CanvasButton has a public OnClick() method (CanvasButton.cs:39)
    /// that performs the action after a debounce check.
    ///
    /// In EditMode, Time.time == 0 and initial _lastClickTime == -1f, so
    /// 0 - (-1) = 1.0 >= 0.5 passes the debounce. The first click should work.
    /// </summary>
    [UnityTest]
    public IEnumerator CanvasButtonClickSimulation()
    {
      yield return Initialize();

      var fakeActionService = SetUpFakeActionService();

      // Create a CanvasButton with required serialized fields
      var canvasButton = CreateSceneObject<CanvasButton>(b =>
      {
        var canvasGroupGo = new GameObject("CanvasGroup");
        canvasGroupGo.transform.SetParent(b.transform);
        b._canvasGroup = canvasGroupGo.AddComponent<CanvasGroup>();
        var textGo = new GameObject("ButtonText");
        textGo.transform.SetParent(b.transform);
        b._text = textGo.AddComponent<TextMeshProUGUI>();
      });

      // Set a view with a non-null action
      var testAction = new OnClickUnion { Enum = GameActionEnum.NoOp };
      canvasButton.SetView(new ButtonView { Label = "Test Canvas", Action = testAction });

      // Call OnClick directly
      canvasButton.OnClick();

      Assert.That(
        fakeActionService.PerformedActions.Count,
        Is.GreaterThan(0),
        "CanvasButton.OnClick() should record an action in FakeActionService"
      );
    }

    // -- Test 5: Displayable Hover Simulation --

    /// <summary>
    /// Verifies that the IInputProvider injection pathway works for simulating
    /// 3D Displayable hover events. The InputService.HandleDisplayableHover()
    /// method (InputService.cs:149) drives hover detection when
    /// IsPointerPressed() is false.
    ///
    /// A test subclass (HoverRecordingDisplayable) records whether hover
    /// methods were called, since the base Displayable has empty virtual
    /// implementations.
    /// </summary>
    [UnityTest]
    public IEnumerator DisplayableHoverSimulation_ViaIInputProvider()
    {
      yield return Initialize();

      // Create a HoverRecordingDisplayable
      var displayable = CreateSceneObject<HoverRecordingDisplayable>();

      // Set up fake input provider
      var fakeInput = new FakeInputProvider();
      Registry.InputService.InputProvider = fakeInput;

      // Hover requires IsPointerPressed() == false and a target
      fakeInput.Pressed = false;
      fakeInput.HoverTarget = displayable;

      // Drive InputService to process hover
      Registry.InputService.Update();

      Assert.IsTrue(
        displayable.HoverStartCalled,
        "MouseHoverStart() should be called when IInputProvider returns a hover target"
      );

      // Drive another update to test MouseHover (continued hover on same object)
      Registry.InputService.Update();

      Assert.IsTrue(
        displayable.HoverCalled,
        "MouseHover() should be called on subsequent frames while hovering the same object"
      );

      // Remove hover target to trigger MouseHoverEnd
      fakeInput.HoverTarget = null;
      Registry.InputService.Update();

      Assert.IsTrue(
        displayable.HoverEndCalled,
        "MouseHoverEnd() should be called when hover target is removed"
      );
    }
  }
}
