import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldChipsInput } from './field-chips-input';

describe('FieldChipsInput', () => {
  let component: FieldChipsInput;
  let fixture: ComponentFixture<FieldChipsInput>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [FieldChipsInput]
    })
    .compileComponents();

    fixture = TestBed.createComponent(FieldChipsInput);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
