import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldTimepicker } from './field-timepicker';

describe('FieldTimepicker', () => {
  let component: FieldTimepicker;
  let fixture: ComponentFixture<FieldTimepicker>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [FieldTimepicker]
    })
    .compileComponents();

    fixture = TestBed.createComponent(FieldTimepicker);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
