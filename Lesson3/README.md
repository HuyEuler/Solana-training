# Lesson 3: Working with Accounts and Data Storage

---

## 💸 Rent (Phí lưu trữ dữ liệu)

Trên Solana, **mỗi account lưu trữ dữ liệu** cần trả một khoản phí gọi là **rent**.

### 🔑 Quy tắc Rent:
- Rent = phí SOL tính theo byte dữ liệu/năm.
- Hiện nay, để giữ dữ liệu vĩnh viễn, bạn phải trả **2 năm rent** để account đạt trạng thái **rent-exempt**.
- Nếu không đạt rent-exempt, account có thể bị reclaim khi thiếu SOL.

📦 Ví dụ: Tính rent cho account 32 bytes:

```bash
solana rent 32
```
## Anchor's account macro 


