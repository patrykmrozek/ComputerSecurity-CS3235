# Expected Sample Output



### Payload 1: Buffer Overflow


```bash
==============================Final Database State:===========================================
User: CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC, ID: 1, Email: mallory@nus.edu.sg, Inactivity: 0  Password = malloryisnotevil
User: Alice, ID: 2, Email: alice@nus.edu.sg, Inactivity: 0  Password = aliceinthewonderland
User: Bob, ID: 3, Email: bob@nus.edu.sg, Inactivity: 0  Password = bobthebuilder
User: Eve, ID: 4, Email: eve@nus.edu.sg, Inactivity: 0  Password = eve4ever
==============================================================================================
```


### Payload 2: Use After Free


```bash
==============================Final Database State:===========================================
User: Alice, ID: 2, Email: alice@nus.edu.sg, Inactivity: 1  Password = aliceinthewonderland
User: Bob, ID: 3, Email: bob@nus.edu.sg, Inactivity: 1  Password = bobthebuilder
==============================================================================================
```

### Payload 3: Double Free

```bash
==============================Final Database State:===========================================
User: Alice, ID: 2, Email: alice@nus.edu.sg, Inactivity: 1  Password = aliceinthewonderland
User: Bob, ID: 3, Email: bob@nus.edu.sg, Inactivity: 1  Password = bobthebuilder
User: Charlie, ID: 5, Email: charlie@nus.edu.sg, Inactivity: 0  Password = charlieandthechocolatefactory
User: Bruce, ID: 6, Email: bruce@nus.edu.sg, Inactivity: 0  Password = iambatman
User: Joker, ID: 7, Email: joker@nus.edu.sg, Inactivity: 0  Password = whysoserious
==============================================================================================
```