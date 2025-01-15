import { ComponentFixture, TestBed } from '@angular/core/testing';

import { CalendarHeaderComponent } from './calendar-header.component';

describe('CalendarHeaderComponent', () => {
  let component: CalendarHeaderComponent<Date>;
  let fixture: ComponentFixture<CalendarHeaderComponent<Date>>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [CalendarHeaderComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(CalendarHeaderComponent<Date>);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
