# create-ebook Skill

สร้างหนังสือ Ebook ทั้งรูปแบบ PDF และ EPUB จาก Markdown หลายๆ ไฟล์ในโฟลเดอร์เดียว

## วิธีใช้

```bash
create-ebook <โฟลเดอร์ที่เก็บไฟล์ md> <ชื่อไฟล์ผลลัพธ์>
```

### ตัวอย่าง
```bash
create-ebook @cli-made-easy-drafts/ my-awesome-book
```

- จะทำการรวมไฟล์ .md ทั้งหมดในโฟลเดอร์ (เรียงตามชื่อไฟล์)
- ใส่ตัวแบ่งหน้า (Page Break) ให้ทุกบท
- สร้างไฟล์ .pdf และ .epub ออกมาให้ทันทีเลยค่ะ!
