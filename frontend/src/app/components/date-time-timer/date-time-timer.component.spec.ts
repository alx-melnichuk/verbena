import { ComponentFixture, TestBed } from '@angular/core/testing';

import { DateTimeTimerComponent } from './date-time-timer.component';

describe('DateTimeTimerComponent', () => {
  let component: DateTimeTimerComponent;
  let fixture: ComponentFixture<DateTimeTimerComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [DateTimeTimerComponent]
    });
    fixture = TestBed.createComponent(DateTimeTimerComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
