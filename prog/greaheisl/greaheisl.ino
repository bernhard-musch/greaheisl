#include "Arduino_LED_Matrix.h"
// My RTC of the Arduino Wifi Rev4 is totally useless!
// About an hour deviation per day!
// #include "RTC.h"
#include "greaheisl_lib.h"
#include <cstdlib>


ArduinoLEDMatrix matrix;

const pin_size_t INPUT_BUTTON_PINS[NUM_BUTTONS] = {
  2, // Exit
  3, // Prev
  4, // Next
  5  // Enter
};

const pin_size_t OUTPUT_RELAY_PINS[NUM_RELAYS] = {
  6, 
  7, 
  8, 
  9  
};

void *my_aligned_alloc(size_t align, size_t size) {
  Serial.print("alloc ");
  Serial.print(size);
  Serial.print(" aligned ");
  Serial.println(align);
  return std::malloc(size);
}

GreaheislExecutor *greaheisl;
unsigned long rtc_millis_offset = 0;

void callback_get_rtc(RtcTime *rtc_time) {
  /* Arduino RTC is useless, use millis() as a preliminary substitute
  RTCTime currenttime;
  RTC.getTime(currenttime);
  rtc_time->hour = currenttime.getHour();
  rtc_time->minute = currenttime.getMinutes();
  rtc_time->second = currenttime.getSeconds();
  */
  unsigned long seconds = (millis() + rtc_millis_offset)/1000;
  unsigned long minutes = seconds/60;
  unsigned long hours = minutes/60;
  unsigned long days = hours/24;
  rtc_time->second = seconds - minutes*60;
  rtc_time->minute = minutes - hours*60;
  rtc_time->hour = hours - days*24;
}

void callback_set_rtc(const RtcTime* rtc_time) {
  /* Arduino RTC is useless, use millis() as a preliminary substitute
  RTCTime newtime;
  RTC.getTime(newtime);
  newtime.setHour(rtc_time->hour);
  newtime.setMinute(rtc_time->minute);
  newtime.setSecond(rtc_time->second);
  RTC.setTime(newtime);
  */
  unsigned long new_millis = 1000ul * (unsigned long)rtc_time->second
                           + 1000ul * 60ul * (unsigned long)rtc_time->minute
                           + 1000ul * 60ul * 60ul * (unsigned long)rtc_time->hour;
  rtc_millis_offset = new_millis - millis();
}

void callback_set_led_matrix(const uint32_t (*screen)[3]) {
  matrix.loadFrame(*screen);
}

uint8_t callback_get_button_flags() {
  uint8_t result = 0;
  uint8_t mask = 1;
  for (unsigned k=0;k<NUM_BUTTONS;k++) {
    if (digitalRead(INPUT_BUTTON_PINS[k]) == LOW) {
      //button pressed
      result |= mask;
    }
    mask <<= 1;
  }
  return result; 
}

void callback_set_relay_states(const bool (*relay_states)[NUM_RELAYS]) {
  for (unsigned k=0;k<NUM_RELAYS;k++) {
    digitalWrite(OUTPUT_RELAY_PINS[k], (*relay_states)[k] ? HIGH : LOW );  
  }
}

const GreaheislCallbacks callbacks = { callback_get_rtc, callback_set_rtc, callback_set_led_matrix, callback_get_button_flags, callback_set_relay_states };

void setup() {
  Serial.begin(115200);
  matrix.begin();
  //RTC.begin();

  
  for (unsigned k=0;k<NUM_BUTTONS;k++) {
    pinMode(INPUT_BUTTON_PINS[k],INPUT_PULLUP);
  }
  for (unsigned k=0;k<NUM_RELAYS;k++) {
    pinMode(OUTPUT_RELAY_PINS[k],OUTPUT);
    digitalWrite(OUTPUT_RELAY_PINS[k],LOW);
  }
  
  /*
  RTCTime mytime(25, Month::AUGUST, 2022, 14, 37, 00, DayOfWeek::THURSDAY, SaveLight::SAVING_TIME_ACTIVE);
  if(!RTC.isRunning()) {
    RTC.setTime(mytime);
  }
  */
  set_allocator_functions(my_aligned_alloc, std::free);
  Serial.println("Starting setup.");
  greaheisl = greaheisl_init(&callbacks,millis());
  Serial.println("Done setting up.");
}

void loop() {
  //Serial.println("Hello world!");
  //delay(500);
  unsigned long delay_time = greaheisl_step(greaheisl,millis(),1);
  delay_time = min(delay_time,10); // never wait longer than 20 milliseconds, because we need to check for button state change
  //Serial.println("step");
  delay(delay_time);
}

