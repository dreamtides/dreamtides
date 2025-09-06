#nullable enable

using System.Runtime.InteropServices;
using System.Text;
using Dreamtides.Schema;
using Dreamtides.Utils;
using Newtonsoft.Json;

static class Plugin
{
    const int BufferSize = 10_000_000;

    public static ConnectResponse Connect(ConnectRequest request)
    {
        var serialized = JsonConvert.SerializeObject(request, Converter.Settings);
        var encoded = Encoding.UTF8.GetBytes(serialized);

        byte[] response = new byte[BufferSize];
        int responseLength = Errors.CheckNonNegative(
            dreamtides_connect(encoded, encoded.Length, response, BufferSize));
        var json = Encoding.UTF8.GetString(response, 0, responseLength);
        var deserialized = JsonConvert.DeserializeObject<ConnectResponse>(json, Converter.Settings);
        return Errors.CheckNotNull(deserialized, "Error deserializing connect response");
    }

    public static PerformActionResponse PerformAction(PerformActionRequest request)
    {
        var serialized = JsonConvert.SerializeObject(request, Converter.Settings);
        var encoded = Encoding.UTF8.GetBytes(serialized);

        byte[] response = new byte[BufferSize];
        int responseLength = Errors.CheckNonNegative(
            dreamtides_perform_action(encoded, encoded.Length, response, BufferSize));
        var json = Encoding.UTF8.GetString(response, 0, responseLength);
        var deserialized = JsonConvert.DeserializeObject<PerformActionResponse>(json, Converter.Settings);
        return Errors.CheckNotNull(deserialized, "Error deserializing action response");
    }

    public static bool HasPendingUpdates()
    {
        return dreamtides_has_updates();
    }

    public static PollResponse Poll(PollRequest request)
    {
        var serialized = JsonConvert.SerializeObject(request, Converter.Settings);
        var encoded = Encoding.UTF8.GetBytes(serialized);

        byte[] response = new byte[BufferSize];
        int responseLength = Errors.CheckNonNegative(
            dreamtides_poll(encoded, encoded.Length, response, BufferSize));
        var json = Encoding.UTF8.GetString(response, 0, responseLength);
        var deserialized = JsonConvert.DeserializeObject<PollResponse>(json, Converter.Settings);
        return Errors.CheckNotNull(deserialized, "Error deserializing poll response");
    }

    public static void Log(ClientLogRequest request)
    {
        var serialized = JsonConvert.SerializeObject(request, Converter.Settings);
        var encoded = Encoding.UTF8.GetBytes(serialized);
        Errors.CheckNonNegative(dreamtides_log(encoded, encoded.Length));
    }

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("plugin")]
#endif
    public static extern int dreamtides_connect(
      byte[] request,
      int requestLength,
      [Out] byte[] response,
      int responseLength);

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("plugin")]
#endif
    public static extern int dreamtides_perform_action(
      byte[] request,
      int requestLength,
      [Out] byte[] response,
      int responseLength);

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("plugin")]
#endif
    public static extern bool dreamtides_has_updates();

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("plugin")]
#endif
    public static extern int dreamtides_poll(
      byte[] request,
      int requestLength,
      [Out] byte[] response,
      int responseLength);

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("plugin")]
#endif
    public static extern int dreamtides_log(
      byte[] request,
      int requestLength
    );
}