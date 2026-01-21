// TODO: Heartbeat mechanism for auto mode daemon
//
// Responsibilities:
// - Background thread updating .llmc/auto.heartbeat every 5 seconds
// - Atomic file writes (temp file + rename)
// - Include timestamp and instance_id in heartbeat file
