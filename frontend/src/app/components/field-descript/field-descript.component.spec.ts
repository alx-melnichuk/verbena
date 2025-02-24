import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldDescriptComponent } from './field-descript.component';

describe('FieldDescriptComponent', () => {
  let component: FieldDescriptComponent;
  let fixture: ComponentFixture<FieldDescriptComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [FieldDescriptComponent]
    });
    fixture = TestBed.createComponent(FieldDescriptComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
