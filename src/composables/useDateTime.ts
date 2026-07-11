import { ref, onMounted, onUnmounted } from "vue";
import dayjs from "dayjs";
import "dayjs/locale/zh-cn";

dayjs.locale("zh-cn");

const weekDayMap = ["星期日", "星期一", "星期二", "星期三", "星期四", "星期五", "星期六"];

interface DateTimeInfo {
  date: string;       // 2026-07-11
  time: string;       // 14:30:05
  weekDay: string;    // 星期六
  full: string;       // 2026年7月11日 星期六 14:30
}

export function useDateTime() {
  const dateTime = ref<DateTimeInfo>(getDateTimeInfo());

  function getDateTimeInfo(): DateTimeInfo {
    const now = dayjs();
    return {
      date: now.format("YYYY-MM-DD"),
      time: now.format("HH:mm:ss"),
      weekDay: weekDayMap[now.day()],
      full: now.format("YYYY年M月D日") + " " + weekDayMap[now.day()] + " " + now.format("HH:mm"),
    };
  }

  let timer: ReturnType<typeof setInterval> | null = null;

  onMounted(() => {
    timer = setInterval(() => {
      dateTime.value = getDateTimeInfo();
    }, 1000);
  });

  onUnmounted(() => {
    if (timer) clearInterval(timer);
  });

  return { dateTime };
}
