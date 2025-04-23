# 📘 Lesson 4: Wrting and execute instructions
## 🧱 1. Instruction Handler 
Là nơi xử lý logic của contract khi người dùng gửi giao dịch đến.  
Mỗi hàm xử lý logic chương trình gọi là một **instruction handler**.  
Ví dụ : 
```
#[program]
pub mod my_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, data: u64) -> Result<()> {
        let account = &mut ctx.accounts.my_account;
        account.data = data;
        Ok(())
    }
}
```

## 2. Context Constraints và validate accounts 
Mỗi **instruction** sẽ có một **Context struct** tương ứng (ví dụ Initialize). Struct này định nghĩa các tài khoản được yêu cầu trong instruction đó và có thể thêm ràng buộc (constraints) để đảm bảo tính hợp lệ của các tài khoản.  
Ví dụ :
```
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 8)]
    pub my_account: Account<'info, MyData>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
```

## Một vài constraints thông dụng 
---
**Signer account**
```
#[account(signer)]

#[account(signer @ <custom_error>)]
```
Checks the given account signed the transaction.

---
**Mutable account**
```
#[account(mut)]

#[account(mut @ <custom_error>)]
```
Checks the given account is mutable.

---
**Init account**
```
#[account(
    init, 
    payer = <target_account>, 
    space = <num_bytes>
    )]
```
Creates the account via a CPI to the system program and initializes it (sets its account discriminator). The annotated account is required to sign for this instruction unless `seeds` is provided.

---


## 3. Custom Errors 
Anchor cho phép định nghĩa các lỗi khi gặp các tình huống đặc biệt, sai logic.   
Ví dụ :
```
#[error_code]
pub enum MyError {
    #[msg("Giá trị không hợp lệ")]
    InvalidValue,
}
```
Sử dụng trong instruction :
```
if data > 100 {
    return Err(MyError::InvalidValue.into());
}
```


## 📚 References

- [Derive Macro Accounts](https://docs.rs/anchor-lang/latest/anchor_lang/derive.Accounts.html)
- [Account Data Matching](https://solana.com/vi/developers/courses/program-security/account-data-matching)

---