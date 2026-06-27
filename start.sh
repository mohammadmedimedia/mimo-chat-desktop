#!/bin/bash
# MiMo Chat - Script راه‌اندازی
# این اسکریپت سرور mimo و رابط وب را راه‌اندازی می‌کند

echo "=== MiMo Chat ==="
echo "در حال راه‌اندازی..."

# بررسی آیا mimo نصب است
if ! command -v mimo &> /dev/null; then
    echo "خطا: mimo یافت نشد. لطفاً ابتدا آن را نصب کنید:"
    echo "  curl -fsSL https://mimo.xiaomi.com/install | bash"
    exit 1
fi

# کشتن سرور قبلی اگر در حال اجراست
pkill -f "mimo serve" 2>/dev/null
sleep 1

# راه‌اندازی سرور mimo
echo "راه‌اندازی سرور mimo..."
mimo serve --port 3000 --pure > /tmp/mimo-serve.log 2>&1 &
MPID=$!

# انتظار برای آماده شدن سرور
echo "انتظار برای آماده شدن سرور..."
for i in $(seq 1 10); do
    if curl -s --max-time 1 http://127.0.0.1:3000/session > /dev/null 2>&1; then
        echo "سرور آماده است!"
        break
    fi
    sleep 1
done

# بررسی وضعیت سرور
if ! curl -s --max-time 2 http://127.0.0.1:3000/session > /dev/null 2>&1; then
    echo "خطا: سرور راه‌اندازی نشد. لاگ‌ها را بررسی کنید: /tmp/mimo-serve.log"
    exit 1
fi

# راه‌اندازی سرور وب
echo "راه‌اندازی سرور وب..."
cd "$(dirname "$0")"
python3 -m http.server 8080 > /tmp/mimo-web.log 2>&1 &
WEBPID=$!

sleep 2

echo ""
echo "=== آماده است! ==="
echo "رابط وب: http://localhost:8080"
echo "سرور mimo: http://127.0.0.1:3000"
echo ""
echo "برای توقف:"
echo "  kill $MPID $WEBPID"
echo "  یا: pkill -f 'mimo serve'; pkill -f 'http.server 8080'"
echo ""

# باز کردن مرورگر
if command -v xdg-open &> /dev/null; then
    xdg-open http://localhost:8080 2>/dev/null &
elif command -v open &> /dev/null; then
    open http://localhost:8080 2>/dev/null &
fi

# نگه داشتن اسکریپت
echo "برای توقف اسکریپت Ctrl+C را فشار دهید"
wait
