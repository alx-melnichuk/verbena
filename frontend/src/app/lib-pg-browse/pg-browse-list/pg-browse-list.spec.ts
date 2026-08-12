import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgBrowseList } from './pg-browse-list';

describe('PgBrowseList', () => {
  let component: PgBrowseList;
  let fixture: ComponentFixture<PgBrowseList>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgBrowseList]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgBrowseList);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
