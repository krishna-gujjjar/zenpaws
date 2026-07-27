ALTER TABLE messages ADD COLUMN state_lamport_peer TEXT;
ALTER TABLE messages ADD COLUMN state_lamport_counter INTEGER;

UPDATE messages
SET state_lamport_peer = lamport_peer,
    state_lamport_counter = lamport_counter;

CREATE INDEX messages_state_lamport
    ON messages(state_lamport_counter, state_lamport_peer);
