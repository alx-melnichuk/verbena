import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldMessage } from './field-message';

describe('FieldMessage', () => {
  let component: FieldMessage;
  let fixture: ComponentFixture<FieldMessage>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [FieldMessage]
    })
    .compileComponents();

    fixture = TestBed.createComponent(FieldMessage);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
