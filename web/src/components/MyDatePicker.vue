<script setup lang="ts">
import { NDatePicker } from "naive-ui";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import {
  getStartAndEndTimestampToday,
  getStartAndEndTimestampCurrentMonth,
  dateRangeConvertBackend,
} from "../util";

const props = defineProps<{
  date_start?: number;
  date_end?: number;
}>();
defineEmits<{
  (e: "update:date_start", v?: number): void;
  (e: "update:date_end", v?: number): void;
  (e: "confirm", start?: number, end?: number): void;
}>();

const { t, locale } = useI18n();

// A function that takes a day name and returns an array of start and end time
function getDayRange(
  day:
    | "Sunday"
    | "Monday"
    | "Tuesday"
    | "Wednesday"
    | "Thursday"
    | "Friday"
    | "Saturday"
) {
  // Create a new date object for the current date
  let date = new Date();
  // Get the current day of the week as a number (0-6)
  let currentDay = date.getDay();
  // Get the index of the target day in an array of day names
  let dayNames = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
  ];
  let targetDay = dayNames.indexOf(day);
  // Calculate the difference between the current day and the target day
  let diff = targetDay - currentDay;
  // Set the date to the target day by adding the difference
  date.setDate(date.getDate() + diff);
  // Create a new date object for the start time of the target day
  let startTime = new Date(date);
  // Set the hours, minutes, seconds and milliseconds to zero
  startTime.setHours(0, 0, 0, 0);
  // Create a new date object for the end time of the target day
  let endTime = new Date(date);
  // Set the hours, minutes, seconds and milliseconds to the last moment of the day
  endTime.setHours(23, 59, 59);
  // Return an array of start and end time
  return [startTime.getTime(), endTime.getTime()];
}

const shortcuts = ref();
const setShortcuts = () =>
  (shortcuts.value = {
    [t("date.currentMonth")]: getStartAndEndTimestampCurrentMonth(),
    [t("date.monday")]: getDayRange("Monday"),
    [t("date.tuesday")]: getDayRange("Tuesday"),
    [t("date.wednesday")]: getDayRange("Wednesday"),
    [t("date.thursday")]: getDayRange("Thursday"),
    [t("date.friday")]: getDayRange("Friday"),
    [t("date.saturday")]: getDayRange("Saturday"),
    [t("date.sunday")]: getDayRange("Sunday"),
    [t("date.today")]: getStartAndEndTimestampToday(),
  });
setShortcuts();
watch(locale, setShortcuts);

const range = computed<[number, number] | undefined>(() => {
  const arr: [number, number] = [0, Date.now()];
  if (!props.date_start && !props.date_end) {
    return undefined;
  }
  if (props.date_start) {
    arr[0] = props.date_start * 1000;
  }
  if (props.date_end) {
    arr[1] = props.date_end * 1000;
  }
  return arr;
});


</script>

<template>
  <n-date-picker
    @confirm="
      (range) => {
        if (range) {
          const duration = dateRangeConvertBackend(range);
          $emit('confirm', duration[0], duration[1]);
        } else {
          $emit('confirm', undefined, undefined);
        }
      }
    "
    v-bind="$attrs"
    :value="range"
    :shortcuts="shortcuts"
    @update:value="
      (range) => {
        if (range) {
          const duration = dateRangeConvertBackend(range);
          $emit('update:date_start', duration[0]);
          $emit('update:date_end', duration[1]);
        } else {
          $emit('update:date_start', undefined);
          $emit('update:date_end', undefined);
        }
      }
    "
    type="daterange"
    :start-placeholder="t('common.startDate')"
    :end-placeholder="t('common.endDate')"
    clearable
    @clear="
      () => {
        $emit('update:date_start', undefined);
        $emit('update:date_end', undefined);
        $emit('confirm');
      }
    "
  />
</template>
