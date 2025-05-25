<p style="text-align: center;font-size: 40px;font-weight: 900">Distributed File System</p>

<img 
    style="display: block; 
           margin-left: auto;
           margin-right: auto;
           width: 80%;"
    src="documents/diagram.png" 
    alt="System diagram">
</img>

# 1. Features

System supports file storing and auto-backup among different nodes. The coordination and communication protocol are tailored.

- Heartbeat
- File read/write
- State syncrhonization
- Auto-replication when a node is down

# 2. Start

Start DNS

```bash
./dfs dns
```

Start Master

```bash
./dfs master
```

# Coordination

As adding a node to system, during start-up phase, at least one 1 ip of currently active Node
