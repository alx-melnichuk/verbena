import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamCalendarComponent } from './panel-stream-calendar.component';

describe('PanelStreamCalendarComponent', () => {
  let component: PanelStreamCalendarComponent;
  let fixture: ComponentFixture<PanelStreamCalendarComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelStreamCalendarComponent]
    });
    fixture = TestBed.createComponent(PanelStreamCalendarComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
