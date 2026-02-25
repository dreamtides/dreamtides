#nullable enable

using System.Collections.Generic;
using Newtonsoft.Json;

namespace Abu
{
  /// <summary>
  /// Incoming command from the daemon.
  /// </summary>
  public class AbuCommand
  {
    [JsonProperty("id")]
    public string Id { get; set; } = "";

    [JsonProperty("command")]
    public string Command { get; set; } = "";

    [JsonProperty("params")]
    public Newtonsoft.Json.Linq.JObject? Params { get; set; }
  }

  /// <summary>
  /// Outgoing response to the daemon.
  /// </summary>
  public class AbuResponse
  {
    [JsonProperty("id")]
    public string Id { get; set; } = "";

    [JsonProperty("success")]
    public bool Success { get; set; }

    [JsonProperty("data", NullValueHandling = NullValueHandling.Ignore)]
    public object? Data { get; set; }

    [JsonProperty("error", NullValueHandling = NullValueHandling.Ignore)]
    public string? Error { get; set; }
  }

  /// <summary>
  /// A node in the scene tree, used in snapshot responses.
  /// </summary>
  public class AbuSceneNode
  {
    [JsonProperty("role")]
    public string Role { get; set; } = "";

    [JsonProperty("label", NullValueHandling = NullValueHandling.Ignore)]
    public string? Label { get; set; }

    [JsonProperty("interactive")]
    public bool Interactive { get; set; }

    [JsonProperty("children")]
    public List<AbuSceneNode> Children { get; set; } = new List<AbuSceneNode>();
  }

  /// <summary>
  /// Typed parameters for the snapshot command.
  /// </summary>
  public class SnapshotParams
  {
    [JsonProperty("interactive")]
    public bool? Interactive { get; set; }

    [JsonProperty("compact")]
    public bool? Compact { get; set; }

    [JsonProperty("maxDepth")]
    public int? MaxDepth { get; set; }

    [JsonProperty("effectLogs")]
    public bool? EffectLogs { get; set; }
  }

  /// <summary>
  /// Typed parameters for the click and hover commands.
  /// </summary>
  public class RefParams
  {
    [JsonProperty("ref")]
    public string Ref { get; set; } = "";

    [JsonProperty("effectLogs")]
    public bool? EffectLogs { get; set; }
  }

  /// <summary>
  /// Typed parameters for the drag command.
  /// </summary>
  public class DragParams
  {
    [JsonProperty("source")]
    public string Source { get; set; } = "";

    [JsonProperty("target", NullValueHandling = NullValueHandling.Ignore)]
    public string? Target { get; set; }

    [JsonProperty("effectLogs")]
    public bool? EffectLogs { get; set; }
  }

  /// <summary>
  /// A ref entry describing an interactive node in the formatted snapshot.
  /// </summary>
  public class SnapshotRef
  {
    [JsonProperty("role")]
    public string Role { get; set; } = "";

    [JsonProperty("name")]
    public string Name { get; set; } = "";
  }

  /// <summary>
  /// Snapshot response data containing formatted ARIA-style text and ref mappings.
  /// </summary>
  public class SnapshotData
  {
    [JsonProperty("snapshot")]
    public string Snapshot { get; set; } = "";

    [JsonProperty("refs")]
    public Dictionary<string, SnapshotRef> Refs { get; set; } =
      new Dictionary<string, SnapshotRef>();

    [JsonProperty("history", NullValueHandling = NullValueHandling.Ignore)]
    public List<string>? History { get; set; }

    [JsonProperty("effectLogs", NullValueHandling = NullValueHandling.Ignore)]
    public List<string>? EffectLogs { get; set; }
  }

  /// <summary>
  /// Combined action + snapshot response data returned after an action settles.
  /// Serializes the action-specific fields (e.g. clicked, hovered) from ActionData
  /// alongside the inherited formatted snapshot text and refs.
  /// </summary>
  public class ActionSnapshotData : SnapshotData
  {
    [JsonExtensionData]
    public Dictionary<string, Newtonsoft.Json.Linq.JToken>? ActionFields { get; set; }

    [JsonIgnore]
    public object? ActionData
    {
      set
      {
        var serialized = Newtonsoft.Json.Linq.JObject.FromObject(value!);
        ActionFields = new Dictionary<string, Newtonsoft.Json.Linq.JToken>();
        foreach (var property in serialized.Properties())
        {
          ActionFields[property.Name] = property.Value;
        }
      }
    }
  }
}
