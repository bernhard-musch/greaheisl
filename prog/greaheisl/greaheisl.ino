// 
// "Sketch" to be compiled and uploaded on the
// Arduino UNO Wifi Rev4 using the Arduino IDE.  
//


// We use the provided library functions
//for the on-board 12 x 8 LED matrix
#include "Arduino_LED_Matrix.h"

// There is an on-board RTC (real time clock) on the
// of the Arduino UNO Wifi Rev4.
// Unfortunately, it did not work for me. I get about
// an hour deviation per day!
// Therefore, I cannot use it. Include file commented out.
// #include "RTC.h"

// This is the interface to our own Rust library provided
// in this repository.
#include "greaheisl_lib.h"

#include <cstdlib>

ArduinoLEDMatrix matrix;

// This defines the pins we use for the input buttons.
const pin_size_t INPUT_BUTTON_PINS[NUM_BUTTONS] = {
  2, // Exit
  3, // Prev
  4, // Next
  5  // Enter
};

// This defines the pins we use for our relay output.
const pin_size_t OUTPUT_RELAY_PINS[NUM_RELAYS] = {
  6, 
  7, 
  8, 
  9  
};

// Opaque handle initialized and used by `greaheisl_lib`
GreaheislExecutor *greaheisl;

// Time offset needed to use the millis() counter
// as a substitute for the real time clock.
unsigned long rtc_millis_offset = 0;

/* -------------------------------------------------- 
 *
 *   Callback functions required by `greaheisl_lib`
 *
 * -------------------------------------------------- */

// dynamic memory allocation
void *my_aligned_alloc(size_t align, size_t size) {
  Serial.print("alloc ");
  Serial.print(size);
  Serial.print(" aligned ");
  Serial.println(align);
  // We ignore the alignment requirement here.
  // Memory blocks allocated using `malloc`
  // are always maximally aligned, so we expect
  // the alignment requirement to be fulfilled
  // anyway.
  // However, to be safe, it would be better to 
  // check if the alignment requirement is really fulfilled. 
  return std::malloc(size);
}

// read the clock
void callback_get_rtc(RtcTime *rtc_time) {
  /* My Arduino RTC is terribly inaccurate; use millis() as a preliminary substitute
  RTCTime currenttime;
  RTC.getTime(currenttime);
  rtc_time->hour = currenttime.getHour();
  rtc_time->minute = currenttime.getMinutes();
  rtc_time->second = currenttime.getSeconds();
  */
  // convert the millisecond information into hours, minutes, seconds
  unsigned long seconds = (millis() + rtc_millis_offset)/1000;
  unsigned long minutes = seconds/60;
  unsigned long hours = minutes/60;
  unsigned long days = hours/24;
  rtc_time->second = seconds - minutes*60;
  rtc_time->minute = minutes - hours*60;
  rtc_time->hour = hours - days*24;
}

// set the clock
void callback_set_rtc(const RtcTime* rtc_time) {
  /* My Arduino RTC is terribly inaccurate; use millis() as a preliminary substitute
  RTCTime newtime;
  RTC.getTime(newtime);
  newtime.setHour(rtc_time->hour);
  newtime.setMinute(rtc_time->minute);
  newtime.setSecond(rtc_time->second);
  RTC.setTime(newtime);
  */
  // compute the offset such that we now get the desired time 
  unsigned long new_millis = 1000ul * (unsigned long)rtc_time->second
                           + 1000ul * 60ul * (unsigned long)rtc_time->minute
                           + 1000ul * 60ul * 60ul * (unsigned long)rtc_time->hour;
  rtc_millis_offset = new_millis - millis();
}

// set the LED bitmap
void callback_set_led_matrix(const uint32_t (*screen)[3]) {
  matrix.loadFrame(*screen);
}

// retrieve flags which buttons are pressed
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

// set the output pins for the relays
void callback_set_relay_states(const bool (*relay_states)[NUM_RELAYS]) {
  for (unsigned k=0;k<NUM_RELAYS;k++) {
    digitalWrite(OUTPUT_RELAY_PINS[k], (*relay_states)[k] ? HIGH : LOW );  
  }
}

// collect all callbacks in a structure we can pass to `greaheisl_lib`
const GreaheislCallbacks callbacks = { callback_get_rtc, callback_set_rtc, callback_set_led_matrix, callback_get_button_flags, callback_set_relay_states };

/* -------------------------------------------------- 
 *
 *   main entry points: setup() and loop()
 *
 * -------------------------------------------------- */


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
  // Make memory allocation available to `greaheisl_lib`.
  // This has to be done before we do anything else with
  // the library!
  set_allocator_functions(my_aligned_alloc, std::free);
  Serial.println("Starting setup.");
  // initialize the library
  greaheisl = greaheisl_init(&callbacks,millis());
  Serial.println("Done setting up.");
}

void loop() {
  // Call the async executor of `greaheisl_lib`.
  // It does some processing, then returns with a request
  // not to wait more than `delay_time` until it gets called again.
  // In fact, we should call it earlier in case of an event,
  // namely if the button state changes.
  //
  // In principle, we could use an interrupt to wait
  // for a button state change. 
  // Probably it would only work for one button.
  // Nevertheless this could be done to save energy.
  // We could define one button that
  // needs to be pressed in order to wake up
  // the system from a power saving mode.
  // This is not implemented, yet.
  unsigned long delay_time = greaheisl_step(greaheisl,millis(),1);
  // never wait longer than 20 milliseconds, because we need to check for button state change
  delay_time = min(delay_time,10); 
  delay(delay_time);
}

