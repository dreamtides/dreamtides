#nullable enable

using Dreamtides.Layout;

namespace Dreamtides.Services
{
  public class CapabilitiesService : Service
  {
    public bool CanPlayCards() => !Registry.Layout.BrowserBackground.IsVisible;

    /// <summary>
    /// Returns true if a card browser is currently open, e.g. to select a card
    /// or view the contents of the void.
    /// </summary>
    public bool AnyBrowserOpen() => Registry.Layout.Browser.IsOpen || Registry.Layout.CardOrderSelector.IsOpen;

    /// <summary>
    /// Can the user currently info zoom a card that exists in the provided
    /// GameContext to display a large format version of the card.
    /// </summary>
    public bool CanInfoZoom(GameContext gameContext)
    {
      if (Registry.DocumentService.IsAnyPanelOpen())
      {
        return false;
      }

      switch (gameContext)
      {
        case GameContext.Browser:
          return Registry.Layout.Browser.Objects.Count > 1 || Registry.Layout.CardOrderSelector.Objects.Count > 1;
        case GameContext.BrowserOverlay:
        case GameContext.Dragging:
          return true;
        case GameContext.Battlefield:
        case GameContext.Hand:
        case GameContext.Stack:
        case GameContext.GameModifiers:
          return !AnyBrowserOpen();
        default:
          return false;
      }
    }
  }
}