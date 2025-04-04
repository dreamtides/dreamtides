#nullable enable

using Dreamcaller.Utils;
using UnityEngine;

namespace Dreamcaller.Layout
{
  /// <summary>
  /// Represents the current location of a GameObject.
  /// </summary>
  public enum GameContext
  {
    // Note: Enum numbers are serialized by Unity and cannot be changed.
    //
    // Keep in sync with the 'Sorting Layers' tab in Unity
    Unspecified = 0,
    Hidden = 1,
    BattlefieldBackground = 4,
    Battlefield = 8,
    PlayerStatus = 12,
    Deck = 15,
    DiscardPile = 20,
    PrimaryActionButton = 23,
    GameModifiers = 28,
    OnScreenStorage = 32,
    Interface = 36,
    CardActivation = 38,
    Stack = 40,
    DrawnCards = 44,
    RevealedCards = 48,
    Hand = 52,
    BrowserBackground = 56,
    Browser = 60,
    Effects = 65,
    UserMessage = 68,
    InfoZoom = 71,
    Dragging = 79,
    BrowserOverlay = 86,
    InterfaceOverlay = 91,
    SplashScreen = 100,
  }

  public static class SortingLayerExtensions
  {
    public static int SortingLayerId(this GameContext context) => context switch
    {
      GameContext.Unspecified => SortingLayer.NameToID("Default"),
      GameContext.Hidden => SortingLayer.NameToID("Hidden"),
      GameContext.BattlefieldBackground => SortingLayer.NameToID("BattlefieldBackground"),
      GameContext.Battlefield => SortingLayer.NameToID("Battlefield"),
      GameContext.PlayerStatus => SortingLayer.NameToID("PlayerStatus"),
      GameContext.Deck => SortingLayer.NameToID("Deck"),
      GameContext.DiscardPile => SortingLayer.NameToID("DiscardPile"),
      GameContext.PrimaryActionButton => SortingLayer.NameToID("PrimaryActionButton"),
      GameContext.GameModifiers => SortingLayer.NameToID("GameModifiers"),
      GameContext.OnScreenStorage => SortingLayer.NameToID("OnScreenStorage"),
      GameContext.Interface => SortingLayer.NameToID("Interface"),
      GameContext.CardActivation => SortingLayer.NameToID("CardActivation"),
      GameContext.Stack => SortingLayer.NameToID("Stack"),
      GameContext.DrawnCards => SortingLayer.NameToID("DrawnCards"),
      GameContext.RevealedCards => SortingLayer.NameToID("RevealedCards"),
      GameContext.Hand => SortingLayer.NameToID("Hand"),
      GameContext.BrowserBackground => SortingLayer.NameToID("BrowserBackground"),
      GameContext.Browser => SortingLayer.NameToID("Browser"),
      GameContext.Effects => SortingLayer.NameToID("Effects"),
      GameContext.UserMessage => SortingLayer.NameToID("UserMessage"),
      GameContext.InfoZoom => SortingLayer.NameToID("InfoZoom"),
      GameContext.Dragging => SortingLayer.NameToID("Dragging"),
      GameContext.BrowserOverlay => SortingLayer.NameToID("BrowserOverlay"),
      GameContext.InterfaceOverlay => SortingLayer.NameToID("InterfaceOverlay"),
      GameContext.SplashScreen => SortingLayer.NameToID("SplashScreen"),
      _ => throw Errors.UnknownEnumValue(context),
    };

    public static bool IsBattlefieldContext(this GameContext context) => context switch
    {
      GameContext.BattlefieldBackground => true,
      GameContext.Battlefield => true,
      GameContext.PlayerStatus => true,
      GameContext.DiscardPile => true,
      GameContext.GameModifiers => true,
      _ => false
    };
  }
}