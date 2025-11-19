import { Keylite } from "./keylite";

const db = new Keylite()

db.open("testdb")

db.putStr("user:1", "tanish")
db.putStr("user:2", "vinayak")
db.putStr("user:3", "samyak")
db.putStr("user:4", "kanav")
db.putStr("user:5", "abhijeet")

const res = db.scanStr("user:2", "user:4")

for (const { key, value } of res) {
  console.log(key.toString(), value.toString())
}
