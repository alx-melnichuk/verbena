import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldDatepicker } from './field-datepicker';

describe('FieldDatepicker', () => {
  let component: FieldDatepicker;
  let fixture: ComponentFixture<FieldDatepicker>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [FieldDatepicker]
    })
    .compileComponents();

    fixture = TestBed.createComponent(FieldDatepicker);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
